use std;


extern crate bip39;

use self::bip39::{Bip39Error};

extern crate rustc_serialize;

use self::rustc_serialize::hex::{FromHexError};

extern crate reqwest;

extern crate serde_json;


#[derive(Debug)]
pub enum CryptoError {
    InvalidKey,
    MissingKey,
    MissingRecovery,
    KeyGenerationFailed,
    DecryptFailed,
    EncryptFailed,
    RetrieveFailed
}

#[allow(unused_variables)]
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


#[derive(Debug)]
pub enum SDAPIError {
    RequestFailed,
    AuthFailed,
    RetryUpload,
    Conflict,
    BlockMissing,
    SessionMissing,
}

impl From<std::io::Error> for SDAPIError {
    fn from(err: std::io::Error) -> SDAPIError {
        match err {
            _ => SDAPIError::RequestFailed
        }
    }
}

impl From<self::reqwest::Error> for SDAPIError {
    fn from(err: self::reqwest::Error) -> SDAPIError {
        match err {
            _ => SDAPIError::RequestFailed
        }
    }
}

impl From<self::serde_json::Error> for SDAPIError {
    fn from(err: self::serde_json::Error) -> SDAPIError {
        match err {
            _ => SDAPIError::RequestFailed
        }
    }
}
