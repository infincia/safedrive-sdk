use std;

// external crate imports

use ::bip39::{Bip39Error};
use ::rustc_serialize::hex::{FromHexError};

#[derive(Debug)]
pub enum KeychainError {
    KeychainUnavailable,
    KeychainItemMissing,
    KeychainInsertFailed(Box<std::error::Error>),
}

impl std::fmt::Display for KeychainError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            KeychainError::KeychainUnavailable => {
                write!(f, "Keychain unavailable")
            },
            KeychainError::KeychainItemMissing => {
                write!(f, "Keychain item missing")
            },
            KeychainError::KeychainInsertFailed(ref err) => {
                write!(f, "Keychain insert failed: {}", err)
            },
        }
    }
}


impl std::error::Error for KeychainError {
    fn description(&self) -> &str {
        match *self {
            KeychainError::KeychainUnavailable => "keychain unavailable",
            KeychainError::KeychainItemMissing => "keychain item missing",
            KeychainError::KeychainInsertFailed(ref err) => err.description(),

        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            KeychainError::KeychainUnavailable => None,
            KeychainError::KeychainItemMissing => None,
            KeychainError::KeychainInsertFailed(ref err) => Some(&**err),
        }
    }
}



#[derive(Debug)]
pub enum CryptoError {
    KeyInvalid,
    KeyMissing,
    RecoveryPhraseInvalid(Bip39Error),
    RecoveryPhraseIncorrect,
    KeyGenerationFailed,
    KeyWrapFailed,
    BlockDecryptFailed,
    BlockEncryptFailed,
    SessionDecryptFailed,
    SessionEncryptFailed,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            CryptoError::KeyInvalid => {
                write!(f, "Invalid key used")
            },
            CryptoError::KeyMissing => {
                write!(f, "Missing key")
            },
            CryptoError::RecoveryPhraseInvalid(ref err) => {
                write!(f, "Recovery phrase incorrect: {}", err)
            },
            CryptoError::RecoveryPhraseIncorrect => {
                write!(f, "Recovery phrase incorrect")
            },
            CryptoError::KeyGenerationFailed => {
                write!(f, "Key generation failed")
            },
            CryptoError::KeyWrapFailed => {
                write!(f, "Key wrapping failed")
            },
            CryptoError::BlockDecryptFailed => {
                write!(f, "Block decrypt failed")
            },
            CryptoError::BlockEncryptFailed => {
                write!(f, "Block encrypt failed")
            },
            CryptoError::SessionDecryptFailed => {
                write!(f, "Session decrypt failed")
            },
            CryptoError::SessionEncryptFailed => {
                write!(f, "Session encrypt failed")
            },
        }
    }
}


impl std::error::Error for CryptoError {
    fn description(&self) -> &str {
        match *self {
            CryptoError::KeyInvalid => "invalid key found",
            CryptoError::KeyMissing => "key missing",
            CryptoError::RecoveryPhraseInvalid(ref err) => err.description(),
            CryptoError::RecoveryPhraseIncorrect => "recovery phrase incorrect",
            CryptoError::KeyGenerationFailed => "key generation failed",
            CryptoError::KeyWrapFailed => "wrapping key failed",
            CryptoError::BlockDecryptFailed => "decrypting block failed",
            CryptoError::BlockEncryptFailed => "encrypting block failed",
            CryptoError::SessionDecryptFailed => "decrypting session failed",
            CryptoError::SessionEncryptFailed => "encrypting session failed",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            CryptoError::KeyInvalid => None,
            CryptoError::KeyMissing => None,
            CryptoError::RecoveryPhraseInvalid(ref err) => Some(err),
            CryptoError::RecoveryPhraseIncorrect => None,
            CryptoError::KeyGenerationFailed => None,
            CryptoError::KeyWrapFailed => None,
            CryptoError::BlockDecryptFailed => None,
            CryptoError::BlockEncryptFailed => None,
            CryptoError::SessionDecryptFailed => None,
            CryptoError::SessionEncryptFailed => None,
        }
    }
}

#[allow(unused_variables)]
impl From<FromHexError> for CryptoError {
    fn from(e: FromHexError) -> CryptoError {
        CryptoError::KeyInvalid
    }
}

impl From<Bip39Error> for CryptoError {
    fn from(e: Bip39Error) -> CryptoError {
        match e {
            Bip39Error::InvalidChecksum => CryptoError::RecoveryPhraseInvalid(e),
            Bip39Error::EntropyUnavailable(_) => CryptoError::KeyGenerationFailed,
            Bip39Error::InvalidKeysize => CryptoError::RecoveryPhraseInvalid(e),
            Bip39Error::InvalidWordLength => CryptoError::RecoveryPhraseInvalid(e),
            Bip39Error::InvalidWord => CryptoError::RecoveryPhraseInvalid(e),
            Bip39Error::LanguageUnavailable => CryptoError::RecoveryPhraseInvalid(e),
        }
    }
}

#[derive(Debug)]
pub enum SDError {
    Internal(String),
    IO(std::io::Error),
    KeychainError(KeychainError),
    RequestFailure(Box<std::error::Error + Send + Sync>),
    NetworkFailure(Box<std::error::Error + Send + Sync>),
    Conflict(SDAPIError),
    BlockMissing,
    SessionMissing,
    RecoveryPhraseIncorrect,
    InsufficientFreeSpace,
    Authentication,
    UnicodeError,
    TokenExpired,
    CryptoError(CryptoError),
    SyncAlreadyInProgress,
    RestoreAlreadyInProgress,
    ExceededRetries(u64),
}

impl std::error::Error for SDError {
    fn description(&self) -> &str {
        match *self {
            SDError::Internal(ref message) => message,
            SDError::IO(ref err) => err.description(),
            SDError::KeychainError(ref err) => err.description(),
            SDError::RequestFailure(ref err) => err.description(),
            SDError::NetworkFailure(ref err) => err.description(),
            SDError::Conflict(ref err) => err.description(),
            SDError::BlockMissing => "block file missing",
            SDError::SessionMissing => "session file missing",
            SDError::RecoveryPhraseIncorrect => "recovery phrase incorrect",
            SDError::InsufficientFreeSpace => "insufficient free space",
            SDError::Authentication => "authentication failed",
            SDError::UnicodeError => "not valid unicode",
            SDError::TokenExpired => "authentication token expired",
            SDError::CryptoError(ref err) => err.description(),
            SDError::SyncAlreadyInProgress => "folder currently being synced",
            SDError::RestoreAlreadyInProgress => "folder currently being restored",
            SDError::ExceededRetries(_) => "exceeded retry count",

        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            SDError::Internal(_) => None,
            SDError::IO(ref err) => Some(err),
            SDError::KeychainError(ref err) => Some(err),
            SDError::RequestFailure(ref err) => Some(&**err),
            SDError::NetworkFailure(ref err) => Some(&**err),
            SDError::Conflict(ref err) => Some(err),
            SDError::BlockMissing => None,
            SDError::SessionMissing => None,
            SDError::RecoveryPhraseIncorrect => None,
            SDError::InsufficientFreeSpace => None,
            SDError::Authentication => None,
            SDError::UnicodeError => None,
            SDError::TokenExpired => None,
            SDError::CryptoError(ref err) => Some(err),
            SDError::SyncAlreadyInProgress => None,
            SDError::RestoreAlreadyInProgress => None,
            SDError::ExceededRetries(_) => None,

        }
    }
}

impl std::fmt::Display for SDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            SDError::Internal(ref message) => {
                write!(f, "{}", message)
            },
            SDError::IO(ref err) => {
                write!(f, "IO failure ({})", err)
            },
            SDError::KeychainError(ref err) => {
                write!(f, "Keychain error ({})", err)
            },
            SDError::RequestFailure(ref err) => {
                write!(f, "API request failed: {}", err)
            },
            SDError::NetworkFailure(ref err) => {
                write!(f, "Network unavailable: {}", err)
            },
            SDError::Conflict(ref err) => {
                write!(f, "API parameter conflict: {}", err)
            },
            SDError::BlockMissing => {
                write!(f, "Block not found on server")
            },
            SDError::SessionMissing => {
                write!(f, "Session not found on server")
            },
            SDError::RecoveryPhraseIncorrect => {
                write!(f, "Recovery phrase incorrect")
            },
            SDError::InsufficientFreeSpace => {
                write!(f, "Insufficient free space")
            },
            SDError::Authentication => {
                write!(f, "Authentication failed")
            },
            SDError::UnicodeError => {
                write!(f, "Invalid Unicode")
            },
            SDError::TokenExpired => {
                write!(f, "SafeDrive authentication token expired")
            },
            SDError::CryptoError(ref err) => {
                write!(f, "Crypto error: {}", err)
            },
            SDError::SyncAlreadyInProgress => {
                write!(f, "Sync already in progress")
            },
            SDError::RestoreAlreadyInProgress => {
                write!(f, "Restore already in progress")
            },
            SDError::ExceededRetries(retries) => {
                write!(f, "Exceeded retry count ({})", retries)
            },
        }
    }
}

impl From<KeychainError> for SDError {
    fn from(e: KeychainError) -> SDError {
        match e {
            _ => SDError::KeychainError(e)
        }
    }
}

impl From<std::io::Error> for SDError {
    fn from(e: std::io::Error) -> SDError {
        match e {
            _ => SDError::IO(e)
        }
    }
}

impl From<CryptoError> for SDError {
    fn from(e: CryptoError) -> Self {
        match e {
            CryptoError::RecoveryPhraseIncorrect => SDError::RecoveryPhraseIncorrect,

            _ =>  SDError::CryptoError(e)
        }
    }
}

impl From<SDAPIError> for SDError {
    fn from(e: SDAPIError) -> Self {
        match e {
            SDAPIError::Internal(err) => SDError::Internal(err),
            SDAPIError::IO(err) => SDError::IO(err),
            SDAPIError::RequestFailed(_) => SDError::RequestFailure(Box::new(e)),
            SDAPIError::NetworkFailure => SDError::NetworkFailure(Box::new(e)),
            SDAPIError::Authentication => SDError::Authentication,
            SDAPIError::BlockMissing => SDError::BlockMissing,
            SDAPIError::SessionMissing => SDError::SessionMissing,
            SDAPIError::Conflict => SDError::Conflict(e),
            // we never actually construct an SDError from this variant so it should never be used,
            // but the compiler requires it to exist or use a catch-all pattern
            SDAPIError::RetryUpload => SDError::RequestFailure(Box::new(e)),
        }
    }
}


#[derive(Debug)]
pub enum SDAPIError {
    Internal(String),
    IO(std::io::Error),
    RequestFailed(Box<std::error::Error + Send + Sync>),
    NetworkFailure,
    Authentication,
    RetryUpload,
    Conflict,
    BlockMissing,
    SessionMissing,
}

impl std::fmt::Display for SDAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            SDAPIError::Internal(ref message) => {
                write!(f, "{}", message)
            },
            SDAPIError::IO(ref err) => {
                write!(f, "{}", err)
            },
            SDAPIError::RequestFailed(ref err) => {
                write!(f, "{}", err)
            },
            SDAPIError::NetworkFailure => {
                write!(f, "Network failure")
            },
            SDAPIError::Authentication => {
                write!(f, "API authentication failed")
            },
            SDAPIError::RetryUpload => {
                write!(f, "Retry upload")
            },
            SDAPIError::Conflict => {
                write!(f, "API parameter conflict")
            },
            SDAPIError::BlockMissing => {
                write!(f, "Block not found on server")
            },
            SDAPIError::SessionMissing => {
                write!(f, "Session not found on server")
            },
        }
    }
}

impl std::error::Error for SDAPIError {
    fn description(&self) -> &str {
        match *self {
            SDAPIError::Internal(ref message) => message,
            SDAPIError::IO(ref err) => err.description(),
            SDAPIError::RequestFailed(ref err) => err.description(),
            SDAPIError::NetworkFailure => "network error",
            SDAPIError::Authentication => "authentication failed",
            SDAPIError::RetryUpload => "retry upload",
            SDAPIError::Conflict => "api parameter conflict",
            SDAPIError::BlockMissing => "block file missing",
            SDAPIError::SessionMissing => "session file missing",

        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            SDAPIError::Internal(_) => None,
            SDAPIError::IO(ref err) => Some(err),
            SDAPIError::RequestFailed(ref err) => Some(&**err),
            SDAPIError::NetworkFailure => None,
            SDAPIError::Authentication => None,
            SDAPIError::RetryUpload => None,
            SDAPIError::Conflict => None,
            SDAPIError::BlockMissing => None,
            SDAPIError::SessionMissing => None,
        }
    }
}

impl From<std::io::Error> for SDAPIError {
    fn from(e: std::io::Error) -> SDAPIError {
        match e {
            _ => SDAPIError::IO(e)
        }
    }
}

impl From<::reqwest::Error> for SDAPIError {
    fn from(e: ::reqwest::Error) -> SDAPIError {
        match e {
            _ => SDAPIError::RequestFailed(Box::new(e))
        }
    }
}

impl From<::serde_json::Error> for SDAPIError {
    fn from(e: ::serde_json::Error) -> SDAPIError {
        match e {
            _ => SDAPIError::RequestFailed(Box::new(e))
        }
    }
}
