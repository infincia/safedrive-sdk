use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(not(target_os = "windows"))]
extern crate interfaces;


extern crate sodiumoxide;

extern crate rustc_serialize;

use self::rustc_serialize::hex::{ToHex};

extern crate bip39;

use self::bip39::{Bip39, Language};

use keys::{KeyType, Key, WrappedKey};
use error::CryptoError;

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
    if let Ok(en0) = interfaces::Interface::get_by_name("en0") {
        if let Some(interface) = en0 {
            if let Ok(mac) = interface.hardware_addr() {
                let mac_string = mac.as_bare_string();
                let to_hash = mac_string + &email;
                let hashed = sodiumoxide::crypto::hash::sha256::hash(to_hash.as_bytes());
                let h = hashed.as_ref().to_hex();
                return Ok(h)
            }
        }
    } else {
        println!("Could not find en0 interface");
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


pub fn get_local_user() -> Result<String, String> {


    let username: String;

    if cfg!(target_os="windows") {
        let output = match Command::new("echo").arg(r"%USERNAME%").output() {
            Ok(out) => { let u = String::from_utf8_lossy(&out.stdout); username = u.to_string() },
            Err(e) => return Err(format!("Failed to get local username: {}", e)),
        };
    } else {
        let output = match Command::new("whoami").output() {
            Ok(out) => { let u = String::from_utf8_lossy(&out.stdout); username = u.to_string()  },
            Err(e) => return Err(format!("Failed to get local username: {}", e)),
        };
    }

    return Ok(username)
}


pub fn generate_keyset() -> Result<(String, WrappedKey, WrappedKey, WrappedKey), CryptoError> {
    // generate a recovery phrase that will be used to encrypt the master key
    let mnemonic_keytype = self::bip39::KeyType::Key128;
    let mnemonic = try!(Bip39::new(&mnemonic_keytype, Language::English, ""));
    println!("Rust<generate_keyset>: phrase: {}", mnemonic.mnemonic);
    let recovery_key = sodiumoxide::crypto::hash::sha256::hash(mnemonic.seed.as_ref());

    // generate a master key and encrypt it with the recovery phrase and static nonce
    // We assign a specific, non-random nonce to use once for each key. Still safe, not reused.
    let master_key_type = KeyType::KeyTypeMaster;
    let master_key = Key::new(master_key_type);
    let master_key_wrapped = try!(master_key.to_wrapped(&recovery_key.as_ref()));

    // generate a main key and encrypt it with the master key and static nonce
    let main_key_type = KeyType::KeyTypeMain;
    let main_key = Key::new(main_key_type);
    let main_key_wrapped = try!(main_key.to_wrapped(&master_key.as_ref()));

    // generate an hmac key and encrypt it with the master key and static nonce
    let hmac_key_type = KeyType::KeyTypeHMAC;
    let hmac_key = Key::new(hmac_key_type);
    let hmac_key_wrapped = try!(hmac_key.to_wrapped(&master_key.as_ref()));

    Ok((mnemonic.mnemonic, master_key_wrapped, main_key_wrapped, hmac_key_wrapped))
}

pub fn decrypt_keyset(phrase: &String, master: WrappedKey, main: WrappedKey, hmac: WrappedKey) -> Result<(Key, Key, Key), CryptoError> {
    let mnemonic = try!(Bip39::from_mnemonic(phrase.clone(), Language::English, "".to_string()));
    let recovery_key = sodiumoxide::crypto::hash::sha256::hash(mnemonic.seed.as_ref());
    let master_key = try!(master.to_key(recovery_key.as_ref()));
    let main_key = try!(main.to_key(master_key.as_ref()));
    let hmac_key = try!(hmac.to_key(master_key.as_ref()));

    Ok((master_key, main_key, hmac_key))
}



#[test]
fn key_wrap_test() {
    let (new_phrase, master_key_wrapped, main_key_wrapped, hmac_key_wrapped) = match generate_keyset() {
        Ok((p, mas, main, hmac)) => (p, mas, main, hmac),
        Err(e) => { assert!(true == false); return }
    };


    match decrypt_keyset(&new_phrase, master_key_wrapped, main_key_wrapped, hmac_key_wrapped) {
        Ok(_) => {},
        Err(e) => { assert!(true == false); return }
    };
}
