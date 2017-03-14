#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg(feature = "keychain")]

use std;

/// external crate imports

use ::keyring::Keyring;


/// internal imports

use ::error::KeychainError;
use ::constants::{account_credential_domain, recovery_key_domain, ssh_credential_domain, token_domain};

/// keychain types

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

/// get


pub fn get_keychain_item(account: &str, service: KeychainService) -> Result<String, KeychainError> {
    let service_name = format!("{}", service);

    let keychain = Keyring::new(&service_name, account);

    let password = keychain.get_password()?;

    Ok(password)
}

/// store

pub fn set_keychain_item(account: &str, service: KeychainService, secret: &str) -> Result<(), KeychainError> {
    let service_name = format!("{}", service);

    let keychain = Keyring::new(&service_name, account);

    keychain.set_password(secret)?;

    Ok(())
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
    let stored_ssh_password = match get_keychain_item("user@safedrive.io", KeychainService::SSHUsername) {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };

    assert!(ssh_username == stored_ssh_password);

}