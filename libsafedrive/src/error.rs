use std;

/// external crate imports

use ::bip39::{Bip39Error};
use ::rustc_serialize::hex::{FromHexError};

use ::keyring::KeyringError;
use reed_solomon::DecoderError;

#[derive(Debug)]
pub enum KeychainError {
    KeychainError(String),
    KeychainUnavailable(String),
    KeychainItemMissing,
    KeychainInsertFailed(String),
    KeychainEncoding(String),
}

impl std::fmt::Display for KeychainError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            KeychainError::KeychainError(ref string) => {
                write!(f, "{}: {}", localized_str!("Keychain error", ""), string)
            },
            KeychainError::KeychainUnavailable(ref string) => {
                write!(f, "{}: {}", localized_str!("Keychain unavailable", ""), string)
            },
            KeychainError::KeychainItemMissing => {
                write!(f, "{}", localized_str!("Keychain item missing", ""))
            },
            KeychainError::KeychainInsertFailed(ref string) => {
                write!(f, "{}: {}", localized_str!("Keychain insert failed", ""), string)
            },
            KeychainError::KeychainEncoding(ref string) => {
                write!(f, "{}: {}", localized_str!("Keychain encoding error", ""), string)
            },
        }
    }
}


impl std::error::Error for KeychainError {
    fn description(&self) -> &str {
        match *self {
            KeychainError::KeychainError(ref string) => string,
            KeychainError::KeychainUnavailable(ref string) => string,
            KeychainError::KeychainItemMissing => localized_str!("keychain item missing", ""),
            KeychainError::KeychainInsertFailed(ref string) => string,
            KeychainError::KeychainEncoding(ref string) => string,

        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            KeychainError::KeychainError(_) => None,
            KeychainError::KeychainUnavailable(_) => None,
            KeychainError::KeychainItemMissing => None,
            KeychainError::KeychainInsertFailed(_) => None,
            KeychainError::KeychainEncoding(_) => None,

        }
    }
}

impl From<KeyringError> for KeychainError {
    fn from(e: KeyringError) -> KeychainError {
        match e {
            KeyringError::Parse(err) => {
                KeychainError::KeychainEncoding(format!("{}", err))
            },
            #[cfg(target_os = "macos")]
            KeyringError::MacOsKeychainError(err) => {
                KeychainError::KeychainError(format!("{}", err))
            },
            KeyringError::NoBackendFound => {
                KeychainError::KeychainUnavailable(format!("no backend found"))
            },
            KeyringError::NoPasswordFound => {
                KeychainError::KeychainItemMissing
            },
            #[cfg(target_os = "linux")]
            KeyringError::SecretServiceError(err) => {
                KeychainError::KeychainError(format!("{}", err))
            },
            #[cfg(target_os = "windows")]
            KeyringError::WindowsVaultError => {
                KeychainError::KeychainError(format!("{}", e))
            }
        }
    }
}


#[derive(Debug)]
pub enum CryptoError {
    KeyInvalid,
    KeyCorrupted,
    RecoveryPhraseInvalid(Box<std::error::Error + Send + Sync>),
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
                write!(f, "{}", localized_str!("Invalid key used", ""))
            },
            CryptoError::KeyCorrupted => {
                write!(f, "{}", localized_str!("Corrupted key", ""))
            },
            CryptoError::RecoveryPhraseInvalid(ref err) => {
                write!(f, "{}: {}", localized_str!("Recovery phrase incorrect", ""), err)
            },
            CryptoError::RecoveryPhraseIncorrect => {
                write!(f, "{}", localized_str!("Recovery phrase incorrect", ""))
            },
            CryptoError::KeyGenerationFailed => {
                write!(f, "{}", localized_str!("Key generation failed", ""))
            },
            CryptoError::KeyWrapFailed => {
                write!(f, "{}", localized_str!("Key wrapping failed", ""))
            },
            CryptoError::BlockDecryptFailed => {
                write!(f, "{}", localized_str!("Block decrypt failed", ""))
            },
            CryptoError::BlockEncryptFailed => {
                write!(f, "{}", localized_str!("Block encrypt failed", ""))
            },
            CryptoError::SessionDecryptFailed => {
                write!(f, "{}", localized_str!("Session decrypt failed", ""))
            },
            CryptoError::SessionEncryptFailed => {
                write!(f, "{}", localized_str!("Session encrypt failed", ""))
            },
        }
    }
}


impl std::error::Error for CryptoError {
    fn description(&self) -> &str {
        match *self {
            CryptoError::KeyInvalid => localized_str!("invalid key found", ""),
            CryptoError::KeyCorrupted => localized_str!("key corrupted", ""),
            CryptoError::RecoveryPhraseInvalid(ref err) => err.description(),
            CryptoError::RecoveryPhraseIncorrect => localized_str!("recovery phrase incorrect", ""),
            CryptoError::KeyGenerationFailed => localized_str!("key generation failed", ""),
            CryptoError::KeyWrapFailed => localized_str!("wrapping key failed", ""),
            CryptoError::BlockDecryptFailed => localized_str!("decrypting block failed", ""),
            CryptoError::BlockEncryptFailed => localized_str!("encrypting block failed", ""),
            CryptoError::SessionDecryptFailed => localized_str!("decrypting session failed", ""),
            CryptoError::SessionEncryptFailed => localized_str!("encrypting session failed", ""),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            CryptoError::KeyInvalid => None,
            CryptoError::KeyCorrupted => None,
            CryptoError::RecoveryPhraseInvalid(ref err) => Some(&**err),
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

#[allow(unused_variables)]
impl From<DecoderError> for CryptoError {
    fn from(e: DecoderError) -> CryptoError {
        match e {
            DecoderError::TooManyErrors => CryptoError::KeyCorrupted,
        }
    }
}

impl From<Bip39Error> for CryptoError {
    fn from(e: Bip39Error) -> CryptoError {
        match e {
            Bip39Error::InvalidChecksum => CryptoError::RecoveryPhraseInvalid(Box::new(e)),
            Bip39Error::EntropyUnavailable(_) => CryptoError::KeyGenerationFailed,
            Bip39Error::InvalidKeysize => CryptoError::RecoveryPhraseInvalid(Box::new(e)),
            Bip39Error::InvalidWordLength => CryptoError::RecoveryPhraseInvalid(Box::new(e)),
            Bip39Error::InvalidWord => CryptoError::RecoveryPhraseInvalid(Box::new(e)),
            Bip39Error::LanguageUnavailable => CryptoError::RecoveryPhraseInvalid(Box::new(e)),
        }
    }
}

#[derive(Debug)]
pub enum SDError {
    Internal(String),
    IO(Box<std::error::Error + Send + Sync>),
    KeychainError(Box<std::error::Error + Send + Sync>),
    RequestFailure(Box<std::error::Error + Send + Sync>),
    NetworkFailure(Box<std::error::Error + Send + Sync>),
    ServiceUnavailable,
    Conflict(Box<std::error::Error + Send + Sync>),
    BlockMissing,
    SessionMissing,
    BlockUnreadable,
    SessionUnreadable,
    RecoveryPhraseIncorrect,
    KeyCorrupted,
    InsufficientFreeSpace,
    Authentication,
    UnicodeError,
    TokenExpired,
    CryptoError(Box<std::error::Error + Send + Sync>),
    SyncAlreadyInProgress,
    RestoreAlreadyInProgress,
    ExceededRetries(u64),
    Cancelled,
    FolderMissing,
}

impl std::error::Error for SDError {
    fn description(&self) -> &str {
        match *self {
            SDError::Internal(ref message) => message,
            SDError::IO(ref err) => err.description(),
            SDError::KeychainError(ref err) => err.description(),
            SDError::RequestFailure(ref err) => err.description(),
            SDError::NetworkFailure(ref err) => err.description(),
            SDError::ServiceUnavailable => localized_str!("service unavailable", ""),
            SDError::Conflict(ref err) => err.description(),
            SDError::BlockMissing => localized_str!("block file missing", ""),
            SDError::SessionMissing => localized_str!("session file missing", ""),
            SDError::BlockUnreadable => localized_str!("block cannot be used", ""),
            SDError::SessionUnreadable => localized_str!("session cannot be used", ""),
            SDError::RecoveryPhraseIncorrect => localized_str!("recovery phrase incorrect", ""),
            SDError::KeyCorrupted => localized_str!("key corrupted", ""),
            SDError::InsufficientFreeSpace => localized_str!("insufficient free space", ""),
            SDError::Authentication => localized_str!("authentication failed", ""),
            SDError::UnicodeError => localized_str!("not valid unicode", ""),
            SDError::TokenExpired => localized_str!("authentication token expired", ""),
            SDError::CryptoError(ref err) => err.description(),
            SDError::SyncAlreadyInProgress => localized_str!("folder currently being synced", ""),
            SDError::RestoreAlreadyInProgress => localized_str!("folder currently being restored", ""),
            SDError::ExceededRetries(_) => localized_str!("exceeded retry count", ""),
            SDError::Cancelled => localized_str!("cancelled sync/restore", ""),
            SDError::FolderMissing => localized_str!("folder missing", ""),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            SDError::Internal(_) => None,
            SDError::IO(ref err) => Some(&**err),
            SDError::KeychainError(ref err) => Some(&**err),
            SDError::RequestFailure(ref err) => Some(&**err),
            SDError::NetworkFailure(ref err) => Some(&**err),
            SDError::ServiceUnavailable => None,
            SDError::Conflict(ref err) => Some(&**err),
            SDError::BlockMissing => None,
            SDError::SessionMissing => None,
            SDError::BlockUnreadable => None,
            SDError::SessionUnreadable => None,
            SDError::RecoveryPhraseIncorrect => None,
            SDError::KeyCorrupted => None,
            SDError::InsufficientFreeSpace => None,
            SDError::Authentication => None,
            SDError::UnicodeError => None,
            SDError::TokenExpired => None,
            SDError::CryptoError(ref err) => Some(&**err),
            SDError::SyncAlreadyInProgress => None,
            SDError::RestoreAlreadyInProgress => None,
            SDError::ExceededRetries(_) => None,
            SDError::Cancelled => None,
            SDError::FolderMissing => None,
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
                write!(f, "{} ({})", localized_str!("IO failure", ""), err)
            },
            SDError::KeychainError(ref err) => {
                write!(f, "{} ({})", localized_str!("Keychain error", ""), err)
            },
            SDError::RequestFailure(ref err) => {
                write!(f, "{}: {}", localized_str!("API request failed", ""), err)
            },
            SDError::NetworkFailure(ref err) => {
                write!(f, "{}: {}", localized_str!("Network unavailable", ""), err)
            },
            SDError::ServiceUnavailable => {
                write!(f, "{}", localized_str!("Service unavailable", ""))
            },
            SDError::Conflict(ref err) => {
                write!(f, "{}: {}", localized_str!("API parameter conflict", ""), err)
            },
            SDError::BlockMissing => {
                write!(f, "{}", localized_str!("Block not found on server", ""))
            },
            SDError::SessionMissing => {
                write!(f, "{}", localized_str!("Session not found on server", ""))
            },
            SDError::BlockUnreadable => {
                write!(f, "{}", localized_str!("Block cannot be used", ""))
            },
            SDError::SessionUnreadable => {
                write!(f, "{}", localized_str!("Session cannot be used", ""))
            },
            SDError::RecoveryPhraseIncorrect => {
                write!(f, "{}", localized_str!("Recovery phrase incorrect", ""))
            },
            SDError::KeyCorrupted => {
                write!(f, "{}", localized_str!("Key corrupted, recover from backup", ""))
            },
            SDError::InsufficientFreeSpace => {
                write!(f, "{}", localized_str!("Insufficient free space", ""))
            },
            SDError::Authentication => {
                write!(f, "{}", localized_str!("Authentication failed", ""))
            },
            SDError::UnicodeError => {
                write!(f, "{}", localized_str!("Invalid Unicode", ""))
            },
            SDError::TokenExpired => {
                write!(f, "{}", localized_str!("SafeDrive authentication token expired", ""))
            },
            SDError::CryptoError(ref err) => {
                write!(f, "{}: {}", localized_str!("Crypto error", ""), err)
            },
            SDError::SyncAlreadyInProgress => {
                write!(f, "{}", localized_str!("Sync already in progress", ""))
            },
            SDError::RestoreAlreadyInProgress => {
                write!(f, "{}", localized_str!("Restore already in progress", ""))
            },
            SDError::ExceededRetries(retries) => {
                write!(f, "{} ({})", localized_str!("Exceeded retry count", ""), retries)
            },
            SDError::Cancelled => {
                write!(f, "{}", localized_str!("Cancelled sync/restore", ""))
            },
            SDError::FolderMissing => {
                write!(f, "{}", localized_str!("Folder missing", ""))
            },
        }
    }
}

impl From<KeychainError> for SDError {
    fn from(e: KeychainError) -> SDError {
        match e {
            _ => SDError::KeychainError(Box::new(e))
        }
    }
}

impl From<std::io::Error> for SDError {
    fn from(e: std::io::Error) -> SDError {
        match e {
            _ => SDError::IO(Box::new(e))
        }
    }
}

impl From<CryptoError> for SDError {
    fn from(e: CryptoError) -> Self {
        match e {
            CryptoError::RecoveryPhraseIncorrect => SDError::RecoveryPhraseIncorrect,

            _ =>  SDError::CryptoError(Box::new(e))
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
            SDAPIError::ServiceUnavailable => SDError::ServiceUnavailable,
            SDAPIError::Authentication => SDError::Authentication,
            SDAPIError::BlockMissing => SDError::BlockMissing,
            SDAPIError::SessionMissing => SDError::SessionMissing,
            SDAPIError::Conflict => SDError::Conflict(Box::new(e)),
            /// we never actually construct an SDError from this variant so it should never be used,
            /// but the compiler requires it to exist or use a catch-all pattern
            SDAPIError::RetryUpload => SDError::RequestFailure(Box::new(e)),
        }
    }
}


#[derive(Debug)]
pub enum SDAPIError {
    Internal(String),
    IO(Box<std::error::Error + Send + Sync>),
    RequestFailed(Box<std::error::Error + Send + Sync>),
    NetworkFailure,
    ServiceUnavailable,
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
                write!(f, "{}", localized_str!("Network failure", ""))
            },
            SDAPIError::ServiceUnavailable => {
                write!(f, "{}", localized_str!("SafeDrive unavailable", ""))
            },
            SDAPIError::Authentication => {
                write!(f, "{}", localized_str!("API authentication failed", ""))
            },
            SDAPIError::RetryUpload => {
                write!(f, "{}", localized_str!("Retry upload", ""))
            },
            SDAPIError::Conflict => {
                write!(f, "{}", localized_str!("API parameter conflict", ""))
            },
            SDAPIError::BlockMissing => {
                write!(f, "{}", localized_str!("Block not found on server", ""))
            },
            SDAPIError::SessionMissing => {
                write!(f, "{}", localized_str!("Session not found on server", ""))
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
            SDAPIError::NetworkFailure => localized_str!("network error", ""),
            SDAPIError::ServiceUnavailable => localized_str!("service unavailable", ""),
            SDAPIError::Authentication => localized_str!("authentication failed", ""),
            SDAPIError::RetryUpload => localized_str!("retry upload", ""),
            SDAPIError::Conflict => localized_str!("api parameter conflict", ""),
            SDAPIError::BlockMissing => localized_str!("block file missing", ""),
            SDAPIError::SessionMissing => localized_str!("session file missing", ""),

        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            SDAPIError::Internal(_) => None,
            SDAPIError::IO(ref err) => Some(&**err),
            SDAPIError::RequestFailed(ref err) => Some(&**err),
            SDAPIError::NetworkFailure => None,
            SDAPIError::ServiceUnavailable => None,
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
            _ => SDAPIError::IO(Box::new(e))
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
