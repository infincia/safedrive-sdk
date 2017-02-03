#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg(feature = "keychain")]

use std;

// external crate imports
#[cfg(target_os = "linux")]
use ::secret_service::SecretService;

#[cfg(target_os = "linux")]
use ::secret_service::EncryptionType;


// internal imports

use ::error::KeychainError;
use ::constants::{account_credential_domain, recovery_key_domain, ssh_credential_domain, token_domain};

// keychain types

pub enum KeychainService {
    Account,
    SSHUsername,
    RecoveryPhrase,
    AuthToken,
}


impl std::fmt::Display for KeychainService {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            KeychainService::Account => {
                write!(f, "{}", account_credential_domain())
            },
            KeychainService::SSHUsername => {
                write!(f, "{}", ssh_credential_domain())
            },
            KeychainService::RecoveryPhrase => {
                write!(f, "{}", recovery_key_domain())
            },
            KeychainService::AuthToken => {
                write!(f, "{}", token_domain())
            },
        }
    }
}

#[cfg(target_os = "macos")]
impl KeychainService {
    fn content_type(&self) -> ::security_framework::item::ItemClass {
        match *self {
            KeychainService::Account => ::security_framework::item::ItemClass::GenericPassword,
            KeychainService::SSHUsername => ::security_framework::item::ItemClass::GenericPassword,
            KeychainService::RecoveryPhrase => ::security_framework::item::ItemClass::GenericPassword,
            KeychainService::AuthToken => ::security_framework::item::ItemClass::GenericPassword,

        }
    }
}

#[cfg(target_os = "linux")]
impl KeychainService {
    fn content_type(&self) -> &str {
        match *self {
            KeychainService::Account => "text/plain",
            KeychainService::SSHUsername => "text/plain",
            KeychainService::RecoveryPhrase => "text/plain",
            KeychainService::AuthToken => "text/plain",
        }
    }
}


#[cfg(target_os = "windows")]
impl KeychainService {
    fn content_type(&self) -> &str {
        match *self {
            KeychainService::Account => "text/plain",
            KeychainService::SSHUsername => "text/plain",
            KeychainService::RecoveryPhrase => "text/plain",
            KeychainService::AuthToken => "text/plain",
        }
    }
}

// get

#[cfg(target_os = "macos")]
pub fn get_keychain_item(account: &str, service: KeychainService) -> Result<String, KeychainError> {
    let mut i = ::security_framework::item::ItemSearchOptions::new();

    i.class(service.content_type());
    i.load_refs(true);
    i.limit(256);

    let results = match i.search() {
        Ok(v) => {
            v
        },
        Err(e) => {
            debug!("{}", e);

            return Err(KeychainError::KeychainItemMissing)
        }
    };

    for result in results {
        match result.reference {
            Some(reference) => {
                println!("ref: {:?}", reference);
            },
            None => {
                return Err(KeychainError::KeychainItemMissing)
            }
        }
    }
    // need generic password support in security-framework to grab these easily
    Err(KeychainError::KeychainUnavailable)
}

#[cfg(target_os = "linux")]
pub fn get_keychain_item(account: &str, service: KeychainService) -> Result<String, KeychainError> {
    let service_name = format!("{}", service);

    // initialize secret service (dbus connection and encryption session)
    let ss = match SecretService::new(EncryptionType::Dh) {
        Ok(ss) => ss,
        Err(e) => {
            debug!("{}", e);

            return Err(KeychainError::KeychainUnavailable)
        }
    };

    // get default collection
    let collection = match ss.get_default_collection() {
        Ok(c) => c,
        Err(e) => {
            debug!("{}", e);

            return Err(KeychainError::KeychainUnavailable)
        }
    };

    // search items by properties
    let search_items = match ss.search_items(
        vec![("account", account), ("service", &service_name)], // properties
    ) {
        Ok(r) => r,
        Err(e) => {
            debug!("{}", e);

            return Err(KeychainError::KeychainItemMissing)
        },

    };


    let item = match search_items.get(0) {
        Some(i) => i,
        None => {
            return Err(KeychainError::KeychainItemMissing)
        },

    };

    // retrieve secret from item
    let secret = match item.get_secret() {
        Ok(s) => s,
        Err(e) => {
            debug!("{}", e);

            return Err(KeychainError::KeychainItemMissing)
        },
    };
    let s = match String::from_utf8(secret) {
        Ok(s) => s,
        Err(e) => return Err(KeychainError::KeychainItemMissing)
    };

    Ok(s)
}

#[cfg(target_os = "windows")]
pub fn get_keychain_item(account: &str, service: KeychainService) -> Result<String, KeychainError> {
    Err(KeychainError::KeychainUnavailable)
}


// store

#[cfg(target_os = "macos")]
pub fn set_keychain_item(account: &str, service: KeychainService, secret: &str) -> Result<(), KeychainError> {
    Err(KeychainError::KeychainUnavailable)
}

#[cfg(target_os = "linux")]
pub fn set_keychain_item(account: &str, service: KeychainService, secret: &str) -> Result<(), KeychainError> {
    let service_name = format!("{}", service);

    // initialize secret service (dbus connection and encryption session)
    let ss = match SecretService::new(EncryptionType::Dh) {
        Ok(ss) => ss,
        Err(e) => {
            debug!("{}", e);

            return Err(KeychainError::KeychainUnavailable)
        }
    };

    // get default collection
    let collection = match ss.get_default_collection() {
        Ok(c) => c,
        Err(e) => {
            debug!("{}", e);

            return Err(KeychainError::KeychainUnavailable)
        }
    };

    //create new item
    match collection.create_item(
        &format!("safedrive"), // label
        vec![("account", account), ("service", &service_name)], // properties
        secret.as_bytes(), //secret
        true, // replace item with same attributes
        service.content_type() // secret content type
    ) {
        Ok(res) => { return Ok(())},
        Err(e) => {
            debug!("{}", e);

            return Err(KeychainError::KeychainInsertFailed(Box::new(e)))
        }
    }
}

#[cfg(target_os = "windows")]
pub fn set_keychain_item(account: &str, service: KeychainService, secret: &str) -> Result<(), KeychainError> {
    Err(KeychainError::KeychainUnavailable)
}





#[test]
fn set_account_password() {
    let password = "password";

    match set_keychain_item("user@safedrive.io", KeychainService::Account, password) {
        Ok(()) => {},
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn get_account_password() {
    let password = "password";

    match set_keychain_item("user@safedrive.io", KeychainService::Account, password) {
        Ok(()) => {},
        Err(_) => { assert!(true == false); return }
    };
    let stored_password = match get_keychain_item("user@safedrive.io", KeychainService::Account) {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };

    assert!(password == stored_password);

}




#[test]
fn set_auth_token() {
    let auth_token = "ABCDEF";

    match set_keychain_item("user@safedrive.io", KeychainService::AuthToken, auth_token) {
        Ok(()) => {},
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn get_auth_token() {
    let auth_token = "ABCDEF";

    match set_keychain_item("user@safedrive.io", KeychainService::AuthToken, auth_token) {
        Ok(()) => {},
        Err(_) => { assert!(true == false); return }
    };
    let stored_auth_token = match get_keychain_item("user@safedrive.io", KeychainService::AuthToken) {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };

    assert!(auth_token == stored_auth_token);

}




#[test]
fn set_recovery_phrase() {
    let recovery_phrase = "alter victory unaware differ negative hole rocket mixture nephew merit iron loud";

    match set_keychain_item("user@safedrive.io", KeychainService::RecoveryPhrase, recovery_phrase) {
        Ok(()) => {},
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn get_recovery_phrase() {
    let recovery_phrase = "alter victory unaware differ negative hole rocket mixture nephew merit iron loud";

    match set_keychain_item("user@safedrive.io", KeychainService::RecoveryPhrase, recovery_phrase) {
        Ok(()) => {},
        Err(_) => { assert!(true == false); return }
    };
    let stored_recovery_phrase = match get_keychain_item("user@safedrive.io", KeychainService::RecoveryPhrase) {
        Ok(rec) => rec,
        Err(_) => { assert!(true == false); return }
    };

    assert!(recovery_phrase == stored_recovery_phrase);
}




#[test]
fn set_ssh_username() {
    let ssh_username = "ABCDEF";

    match set_keychain_item("user@safedrive.io", KeychainService::SSHUsername, ssh_username) {
        Ok(()) => {},
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn get_ssh_username() {
    let ssh_username = "ABCDEF";

    match set_keychain_item("user@safedrive.io", KeychainService::SSHUsername, ssh_username) {
        Ok(()) => {},
        Err(_) => { assert!(true == false); return }
    };
    match get_keychain_item("user@safedrive.io", KeychainService::SSHUsername) {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };
}