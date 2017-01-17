use std;


extern crate bip39;

use self::bip39::{Bip39Error};

extern crate rustc_serialize;

use self::rustc_serialize::hex::{FromHexError};

extern crate reqwest;

extern crate serde_json;


#[derive(Debug)]
pub enum CryptoError {
    KeyInvalid,
    KeyMissing,
    RecoveryPhraseIncorrect,
    KeyGenerationFailed,
    KeyWrapFailed,
    BlockDecryptFailed,
    BlockEncryptFailed,
    SessionDecryptFailed,
    SessionEncryptFailed,
    KeysetRetrieveFailed { embed: SDAPIError },
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &CryptoError::KeyInvalid => {
                write!(f, "Invalid key used")
            },
            &CryptoError::KeyMissing => {
                write!(f, "Missing key")
            },
            &CryptoError::RecoveryPhraseIncorrect => {
                write!(f, "Recovery phrase incorrect")
            },
            &CryptoError::KeyGenerationFailed => {
                write!(f, "Key generation failed")
            },
            &CryptoError::KeyWrapFailed => {
                write!(f, "Key wrapping failed")
            },
            &CryptoError::BlockDecryptFailed => {
                write!(f, "Block decrypt failed")
            },
            &CryptoError::BlockEncryptFailed => {
                write!(f, "Block encrypt failed")
            },
            &CryptoError::SessionDecryptFailed => {
                write!(f, "Session decrypt failed")
            },
            &CryptoError::SessionEncryptFailed => {
                write!(f, "Session encrypt failed")
            },
            &CryptoError::KeysetRetrieveFailed { ref embed } => {
                write!(f, "{}", embed)
            },
        }
    }
}

#[allow(unused_variables)]
impl From<FromHexError> for CryptoError {
    fn from(err: FromHexError) -> CryptoError {
        CryptoError::KeyInvalid
    }
}

impl From<Bip39Error> for CryptoError {
    fn from(err: Bip39Error) -> CryptoError {
        match err {
            Bip39Error::InvalidChecksum => CryptoError::RecoveryPhraseIncorrect,
            Bip39Error::EntropyUnavailable => CryptoError::KeyGenerationFailed,
            Bip39Error::InvalidKeysize => CryptoError::RecoveryPhraseIncorrect,
            Bip39Error::InvalidWordLength => CryptoError::RecoveryPhraseIncorrect,
            Bip39Error::LanguageUnavailable => CryptoError::KeyInvalid
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

impl std::fmt::Display for SDAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &SDAPIError::RequestFailed => {
                write!(f, "API request failed")
            },
            &SDAPIError::AuthFailed => {
                write!(f, "API authentication failed")
            },
            &SDAPIError::RetryUpload => {
                write!(f, "Retry upload")
            },
            &SDAPIError::Conflict => {
                write!(f, "API parameter conflict")
            },
            &SDAPIError::BlockMissing => {
                write!(f, "Block not found on server")
            },
            &SDAPIError::SessionMissing => {
                write!(f, "Session not found on server")
            },
        }
    }
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
