extern crate bip39;

use self::bip39::{Bip39Error};

extern crate rustc_serialize;

use self::rustc_serialize::hex::{FromHexError};


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
