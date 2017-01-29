use std::path::{PathBuf, Path};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

#[cfg(not(target_os = "macos"))]
use std::env;

// external crate imports

use ::rustc_serialize::hex::{ToHex};

use uuid::Uuid;

#[cfg(target_os = "macos")]
use ::objc_foundation::{NSObject, NSString, INSString};

#[cfg(target_os = "macos")]
use ::objc::runtime::{Class};


// internal imports

use ::models::{Configuration, FolderLock};
use ::constants::SDGROUP_NAME;
use ::error::SDError;

pub fn generate_uuid() -> String {
    let sync_uuid = Uuid::new_v4().simple().to_string();

    sync_uuid
}

#[cfg(target_os = "macos")]
pub fn unique_client_hash(email: &str) -> Result<String, String> {
    let interface = "en0";

    if let Ok(hardware) = ::interfaces::Interface::get_by_name(interface) {
        if let Some(interface) = hardware {
            if let Ok(mac) = interface.hardware_addr() {
                let mac_string = mac.as_bare_string();
                let to_hash = mac_string + &email;
                let hash = sha256(to_hash.as_bytes());
                return Ok(hash)
            }
        }
    } else {
        error!("could not find {} interface", interface);
    }
    Err("failed to get mac address".to_string())
}

pub fn get_unique_client_id(local_storage_path: &Path) -> Result<String, SDError> {
    let mut uidp = PathBuf::from(local_storage_path);
    uidp.push(".unique_client_id");

    let mut f = try!(File::open(&uidp));

    let mut unique_client_id = String::new();

    try!(f.read_to_string(&mut unique_client_id));

    Ok(unique_client_id)
}

pub fn set_unique_client_id(unique_client_id: &str, local_storage_path: &Path) -> Result<(), SDError> {
    let mut uidp = PathBuf::from(local_storage_path);
    uidp.push(".unique_client_id");

    let mut f = try!(File::create(&uidp));

    try!(f.write_all(unique_client_id.as_bytes()));

    Ok(())
}

pub fn set_folder_lock_state(path: &Path, state: FolderLock) -> Result<(), SDError> {
    let mut p = PathBuf::from(path);
    p.push(".sdlock");
    let current_lock = match File::open(&p) {
        Ok(mut f) => {
            let mut fbuf = Vec::new();

            match f.read_to_end(&mut fbuf) {
                Ok(_) => FolderLock::from(fbuf),
                Err(_) => FolderLock::Unlocked,
            }
        },
        Err(_) => {
            FolderLock::Unlocked
        }
    };


    match current_lock {
        FolderLock::Unlocked => {
            let mut f = try!(File::create(&p));

            try!(f.write_all(state.as_ref()));
        },
        FolderLock::Sync => {
            return Err(SDError::SyncAlreadyInProgress);
        },
        FolderLock::Restore => {
            return Err(SDError::RestoreAlreadyInProgress);
        },
    }

    Ok(())
}

#[cfg(target_os = "macos")]
#[allow(non_snake_case)]
pub fn get_app_directory(config: &Configuration) -> Result<PathBuf, String> {
    let NSFileManager_cls = Class::get("NSFileManager").unwrap();
    let groupID = NSString::from_str(SDGROUP_NAME);

    let mut storage_path = unsafe {
        let fm: *mut NSObject = msg_send![NSFileManager_cls, new];

        let groupContainerURL: *mut NSObject = msg_send![fm, containerURLForSecurityApplicationGroupIdentifier:&*groupID];

        let groupContainerPath: *mut NSString = msg_send![groupContainerURL, path];

        let s = (*groupContainerPath).as_str();

        let p = PathBuf::from(s);

        let _: () = msg_send![fm, release];
        // these cause a segfault right now, but technically you would release these to avoid a leak
        //let _: () = msg_send![groupContainerURL, release];
        //let _: () = msg_send![groupContainerPath, release];

        p
    };
    match config {
        &Configuration::Staging => storage_path.push("staging"),
        &Configuration::Production => {},
    }
    match fs::create_dir_all(&storage_path) {
        Ok(()) => {},
        Err(_) => {}, //ignore this for the moment, it's primarily going to be the directory existing already
    }
    return Ok(storage_path)

}

#[cfg(not(target_os = "macos"))]
pub fn get_app_directory(config: &Configuration) -> Result<PathBuf, String> {


    let evar: &str;

    if cfg!(target_os="windows") {
        evar = "APPDATA";
    } else {
        // probably linux, use home dir
        evar = "HOME";
    }
    let m = format!("failed to get {} environment variable", evar);
    let path = match env::var(evar) {
        Ok(e) => e,
        Err(_) => { return Err(m) }
    };

    let mut storage_path = PathBuf::from(&path);

    if cfg!(target_os="windows") {
        storage_path.push("SafeDrive");

    } else {
        storage_path.push(".safedrive");
    }

    match config {
        &Configuration::Staging => storage_path.push("staging"),
        &Configuration::Production => {},
    }
    match fs::create_dir_all(&storage_path) {
        Ok(()) => {},
        Err(_) => {}, //ignore this for the moment, it's primarily going to be the directory existing already
    }

    return Ok(storage_path)
}

#[cfg(target_os = "linux")]
pub fn get_openssl_directory() -> Result<PathBuf, ()> {
    let output = match Command::new("openssl")
        .arg("version")
        .arg("-d")
        .output() {
        Ok(o) => o,
        Err(e) => return Err(()),
    };

    let out = output.stdout;

    let mut s = match String::from_utf8(out) {
        Ok(mut s) => s,
        Err(e) => return Err(()),
    };

    let re = match ::regex::Regex::new(r#"OPENSSLDIR: "(.*)""#) {
        Ok(re) => re,
        Err(e) => return Err(()),
    };

    let cap = match re.captures(&s) {
        Some(cap) => cap,
        None => return Err(()),
    };

    let r = &cap[1];

    let path = PathBuf::from(r);

    return Ok(path)
}


pub fn get_current_os() -> &'static str {

    let os: &str;

    if cfg!(target_os="windows") {
        os = "Windows";
    } else if cfg!(target_os="macos") {
        os = "OS X";
    } else if cfg!(target_os="linux") {
        os = "Linux";
    } else {
        os = "Unknown";
    }

    os
}

// crypto helpers

pub fn sha256(input: &[u8]) -> String {
    let hashed = ::sodiumoxide::crypto::hash::sha256::hash(input);
    let h = hashed.as_ref().to_hex();

    h
}

#[cfg(target_os = "linux")]
#[test]
fn openssl_directory_test() {
    match get_openssl_directory() {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };
}



#[test]
fn staging_directory_test() {
    match get_app_directory(&Configuration::Staging) {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn production_directory_test() {
    match get_app_directory(&Configuration::Production) {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };
}