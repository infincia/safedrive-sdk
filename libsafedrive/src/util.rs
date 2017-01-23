use std::path::{PathBuf};
use std::fs;

#[cfg(not(target_os = "macos"))]
use std::env;

// external crate imports

use ::rustc_serialize::hex::{ToHex};

#[cfg(target_os = "macos")]
use ::objc_foundation::{NSObject, NSString, INSString};

#[cfg(target_os = "macos")]
use ::objc::runtime::{Class};


// internal imports

use ::models::Configuration;
use ::constants::SDGROUP_NAME;

#[cfg(target_os = "windows")]
pub fn unique_client_hash(email: &str) -> Result<String, String> {
    Ok("".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn unique_client_hash(email: &str) -> Result<String, String> {
    let interface: &str;
    if cfg!(target_os = "macos") {
        interface = "en0";
    } else {
        interface = "eth0";
    }

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