use std::path::{Path, PathBuf};
use std::fs;
use std::env;

use CONFIGURATION;

#[cfg(unix)]
extern crate interfaces;


extern crate sodiumoxide;

extern crate rustc_serialize;

use self::rustc_serialize::hex::{ToHex};

use ::models::Configuration;

pub fn block_directory(unique_client_id: &str, hmac: &[u8]) -> PathBuf {

    let shard_identifier = hmac[0..1].to_hex().chars().next().unwrap(); //unwrap is safe here, they're always going to be 0-9 or a-f

    let mut storage_dir = Path::new("/storage/").to_owned();
    storage_dir.push(&unique_client_id);
    storage_dir.push(&Path::new("blocks"));
    storage_dir.push(shard_identifier.to_string());
    storage_dir.push(Path::new(&hmac.to_hex()));
    storage_dir
}

pub fn archive_directory(unique_client_id: &str) -> PathBuf {
    let mut storage_dir = Path::new("/storage/").to_owned();
    storage_dir.push(&unique_client_id);
    storage_dir.push(&Path::new("archives"));
    storage_dir
}

#[cfg(target_os = "windows")]
pub fn unique_client_hash(email: &String) -> Result<String, String> {
    Ok("".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn unique_client_hash(email: &String) -> Result<String, String> {
    let interface: &str;
    if cfg!(target_os = "macos") {
        interface = "en0";
    } else {
        interface = "eth0";
    }

    if let Ok(hardware) = interfaces::Interface::get_by_name(interface) {
        if let Some(interface) = hardware {
            if let Ok(mac) = interface.hardware_addr() {
                let mac_string = mac.as_bare_string();
                let to_hash = mac_string + &email;
                let hashed = sodiumoxide::crypto::hash::sha256::hash(to_hash.as_bytes());
                let h = hashed.as_ref().to_hex();
                return Ok(h)
            }
        }
    } else {
        error!("could not find {} interface", interface);
    }
    Err("failed to get mac address".to_string())
}

// not being used at the moment
/*fn folder_to_cfolder(folder: Folder) -> CFolder {
    let s_name = CString::new(folder.name.as_str()).unwrap();
    let s_path = CString::new(folder.path.as_str()).unwrap();
    println!("CFolder found: {:?}:{:?}", &s_name, &s_path);

    CFolder {
        id: folder.id,
        name: s_name,
        path: s_path,
    }
}*/

pub fn get_app_directory(config: &Configuration) -> Result<PathBuf, String> {


    let evar: &str;

    if cfg!(target_os="windows") {
        evar = "APPDATA";
    } else {
        evar = "HOME";
    }
    let m = format!("failed to get {} environment variable", evar);
    let path = match env::var(evar) {
        Ok(e) => e,
        Err(_) => { return Err(m) }
    };

    let mut storage_path = Path::new(&path).to_owned();

    if cfg!(target_os="windows") {
        storage_path.push("SafeDrive");

    } else if cfg!(target_os="macos") {
        storage_path.push("Library");
        storage_path.push("Application Support");
        storage_path.push("SafeDrive");
    } else {
        // probably linux, but either way not one of the others so use home dir
        storage_path.push(".safedrive");
    }

    match config {
        &Configuration::Staging => storage_path.push("staging"),
        &Configuration::Production => {},
    }
    if let Err(e) = fs::create_dir_all(&storage_path) {
        panic!("Failed to create local directories: {}", e);
    }

    return Ok(storage_path)
}


pub fn get_local_user() -> Result<String, String> {
    let evar: &str;

    if cfg!(target_os="windows") {
        evar = "USERNAME";
    } else {
        evar = "USER";
    }

    let m = format!("failed to get {} environment variable", evar);
    let username = match env::var(evar) {
        Ok(e) => e,
        Err(_) => { return Err(m) }
    };

    return Ok(username)
}

#[test]
fn app_directory_test() {
    match get_app_directory() {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn local_username_test() {
    match get_local_user() {
        Ok(u) => u,
        Err(_) => { assert!(true == false); return }
    };
}
