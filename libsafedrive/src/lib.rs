extern crate semver;

#[macro_use]
extern crate bitflags;

extern crate lz4;

#[macro_use] extern crate log;

extern crate chrono;

extern crate number_prefix;

#[macro_use]
mod localized;


mod c_api;
mod core;
mod constants;
mod models;
mod error;
mod keys;
mod util;
mod sdapi;
mod state;
mod binformat;
mod cache;
mod block;
mod session;
mod lock;
mod chunk;
mod oplog;
#[cfg(feature = "keychain")]
mod keychain;

/// public API
///
pub use c_api::*;
pub use core::*;
pub use constants::*;
pub use models::{SyncCleaningSchedule, SyncVersion, Token, RegisteredFolder, AccountStatus, AccountDetails, SoftwareClient};
pub use keys::{Key, Keyset, KeyType};
pub use session::SyncSession;
pub use chunk::{ChunkGenerator, BlockGenerator, BlockGeneratorStats};
#[cfg(feature = "keychain")]
pub use keychain::{KeychainService};





/// external crates

extern crate rustc_serialize;
extern crate libc;
extern crate sodiumoxide;
extern crate libsodium_sys;
extern crate tar;
extern crate rand;
extern crate walkdir;
extern crate cdc;
extern crate bip39;
extern crate serde_json;
extern crate reqwest;
extern crate serde;
extern crate uuid;
extern crate regex;
extern crate parking_lot;
extern crate byteorder;
extern crate blake2_rfc;

#[cfg(feature = "keychain")]
extern crate keyring;

#[cfg(target_os = "macos")]
extern crate interfaces;

#[cfg(target_os = "macos")]
extern crate objc_foundation;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate hyper;

#[macro_use]
extern crate nom;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "locking")]
#[macro_use(defer)]
extern crate scopeguard;

pub static SYNC_VERSION: ::models::SyncVersion = ::models::SyncVersion::Version2;

/// global config, can only be set once at runtime

lazy_static! {
    static ref CONFIGURATION: ::parking_lot::RwLock<constants::Configuration> = ::parking_lot::RwLock::new(constants::Configuration::Production);
}

lazy_static! {
    static ref CHANNEL: ::parking_lot::RwLock<constants::Channel> = {
        let version: &str = env!("CARGO_PKG_VERSION");
        let current_version = ::semver::Version::parse(version).unwrap();

        let stable_version = ::semver::VersionReq::parse(">= 1.0.0").unwrap();

        if stable_version.matches(&current_version) {
            ::parking_lot::RwLock::new(::constants::Channel::Stable)
        } else if !current_version.pre.is_empty() {
            ::parking_lot::RwLock::new(::constants::Channel::Beta)
        } else {
            ::parking_lot::RwLock::new(::constants::Channel::Nightly)
        }

    };
}

lazy_static! {
    static ref CACHE_DIR: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref CLIENT_VERSION: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref USER_AGENT: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref OPERATING_SYSTEM: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref LANGUAGE_CODE: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref CANCEL_LIST: ::parking_lot::RwLock<Vec<String>> = ::parking_lot::RwLock::new(Vec::new());
}








