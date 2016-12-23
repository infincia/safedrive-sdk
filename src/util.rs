use std::path::{Path, PathBuf};
extern crate rustc_serialize;

use self::rustc_serialize::hex::{ToHex, FromHex, FromHexError};
extern crate sodiumoxide;
extern crate interfaces;

extern crate bip39;

use self::bip39::{Bip39, Bip39Error, KeyType, Language};


#[derive(Debug)]
pub enum CryptoError {
    GenerateFailed,
    InvalidKey,
    KeyGenerationFailed,
    DecryptFailed,
    EncryptFailed,
    RetrieveFailed
}

impl From<FromHexError> for CryptoError {
    fn from(err: FromHexError) -> CryptoError {
        CryptoError::InvalidKey
    }
}

impl From<Bip39Error> for CryptoError {
    fn from(err: Bip39Error) -> CryptoError {
        match err {
            Bip39Error::InvalidChecksum => CryptoError::DecryptFailed,
            Bip39Error::EntropyUnavailable => CryptoError::KeyGenerationFailed,
            Bip39Error::InvalidKeysize => CryptoError::InvalidKey,
            Bip39Error::InvalidWordLength => CryptoError::InvalidKey,
            Bip39Error::LanguageUnavailable => CryptoError::DecryptFailed
        }
    }
}


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


pub fn generate_keyset() -> Result<(String, String, String, String), CryptoError> {
    let key_size = sodiumoxide::crypto::secretbox::KEYBYTES;

    // generate a recovery phrase that will be used to encrypt the master key
    let mnemonic_keytype = KeyType::Key128;
    let mnemonic = try!(Bip39::new(&mnemonic_keytype, Language::English, ""));


    let phrase = mnemonic.mnemonic;
    println!("Rust<generate_keyset>: phrase: {}", phrase);
    let seed = sodiumoxide::crypto::hash::sha256::hash(mnemonic.seed.as_ref());
    let hashed_seed = seed.as_ref();

    let recovery_key = sodiumoxide::crypto::secretbox::Key::from_slice(&hashed_seed)
        .expect("Rust<generate_keyset>: failed to get recovery key struct");

    // generate a master key and encrypt it with the recovery phrase and static nonce
    // We assign a specific, non-random nonce to use once for each key. Still safe, not reused.
    let master_key_raw = sodiumoxide::randombytes::randombytes(key_size);
    let master_nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&[1u8; 24]).expect("Rust<generate_keyset>: failed to get master nonce");
    let master_key = sodiumoxide::crypto::secretbox::Key::from_slice(&master_key_raw).expect("Rust<generate_keyset>: failed to get master key struct");
    let master_key_wrapped = sodiumoxide::crypto::secretbox::seal(&master_key_raw, &master_nonce, &recovery_key);

    // generate a main key and encrypt it with the master key and static nonce
    let main_key_raw = sodiumoxide::randombytes::randombytes(key_size);
    let main_nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&[2u8; 24]).expect("Rust<generate_keyset>: failed to get main nonce");
    let main_key_wrapped = sodiumoxide::crypto::secretbox::seal(&main_key_raw, &main_nonce, &master_key);

    // generate an hmac key and encrypt it with the master key and static nonce
    let hmac_key_raw = sodiumoxide::randombytes::randombytes(key_size);
    let hmac_nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&[3u8; 24]).expect("Rust<generate_keyset>: failed to get hmac nonce");
    let hmac_key_wrapped = sodiumoxide::crypto::secretbox::seal(&hmac_key_raw, &hmac_nonce, &master_key);


    Ok((phrase, master_key_wrapped.to_hex(), main_key_wrapped.to_hex(), hmac_key_wrapped.to_hex()))
}

pub fn decrypt_keyset(phrase: &String, master: String, main: String, hmac: String) -> Result<(String, String, String), CryptoError> {

    let mnemonic = try!(Bip39::from_mnemonic(phrase.clone(), Language::English, "".to_string()));

    let seed = sodiumoxide::crypto::hash::sha256::hash(mnemonic.seed.as_ref());
    let hashed_seed = seed.as_ref();

    let recovery_key = sodiumoxide::crypto::secretbox::Key::from_slice(&hashed_seed)
        .expect("Rust<decrypt_keyset>: failed to get recovery key struct");

    // decrypt the master key with the recovery phrase and static nonce
    let master_key_wrapped = try!(master.from_hex());
    let master_nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&[1u8; 24]).expect("Rust<decrypt_keyset>: failed to get master nonce");
    let master_key_raw = match sodiumoxide::crypto::secretbox::open(&master_key_wrapped, &master_nonce, &recovery_key) {
        Ok(k) => k,
        Err(_) => return Err(CryptoError::DecryptFailed)
    };
    let master_key = sodiumoxide::crypto::secretbox::Key::from_slice(&master_key_raw).expect("Rust<decrypt_keyset>: failed to get master key struct");


    // decrypt the main key with the master key and static nonce
    let main_key_wrapped = try!(main.from_hex());
    let main_nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&[2u8; 24]).expect("Rust<decrypt_keyset>: failed to get main nonce");
    let main_key_raw = match sodiumoxide::crypto::secretbox::open(&main_key_wrapped, &main_nonce, &master_key) {
        Ok(k) => k,
        Err(_) => return Err(CryptoError::DecryptFailed)
    };

    // decrypt the hmac key with the master key and static nonce
    let hmac_key_wrapped = try!(hmac.from_hex());
    let hmac_nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&[3u8; 24]).expect("Rust<decrypt_keyset>: failed to get hmac nonce");
    let hmac_key_raw = match sodiumoxide::crypto::secretbox::open(&hmac_key_wrapped, &hmac_nonce, &master_key) {
        Ok(k) => k,
        Err(_) => return Err(CryptoError::DecryptFailed)
    };


    Ok((master_key_raw.to_hex(), main_key_raw.to_hex(), hmac_key_raw.to_hex()))
}
