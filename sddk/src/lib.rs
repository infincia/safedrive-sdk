#![cfg_attr(feature="lint", feature(plugin))]

#![cfg_attr(feature="lint", plugin(clippy))]

#![cfg_attr(feature="webui", feature(plugin))]
#![cfg_attr(feature="webui", plugin(rocket_codegen))]
#![cfg_attr(feature="webui", feature(rustc_attrs))]
#![cfg_attr(feature="webui", feature(custom_attribute))]


#[cfg(feature = "webui")]
extern crate pulldown_cmark;
#[cfg(feature = "webui")]
#[macro_use] extern crate tera;
#[cfg(feature = "webui")]
extern crate rocket;
#[cfg(feature = "webui")]
extern crate rocket_contrib;


extern crate semver;

#[macro_use]
extern crate bitflags;

extern crate lz4;

#[macro_use]
extern crate log;

extern crate simplelog;

extern crate chrono;

extern crate chrono_humanize;

extern crate number_prefix;

extern crate reed_solomon;

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
mod keychain;
#[cfg(feature = "sessionfs")]
mod sessionfs;
mod sdlog;
#[cfg(feature = "webui")]
mod webui;

/// public API
///
pub use c_api::*;
pub use core::*;
pub use constants::*;
pub use error::SDError;
pub use models::{SyncCleaningSchedule, SyncVersion, Token, RegisteredFolder, AccountStatus, AccountDetails, SoftwareClient};
pub use keys::{Key, Keyset, KeyType};
pub use session::SyncSession;
pub use chunk::{ChunkGenerator, BlockGenerator, BlockGeneratorStats};
pub use keychain::KeychainService;

#[cfg(feature = "sessionfs")]
pub use sessionfs::*;




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
extern crate time;

extern crate keyring;

#[cfg(target_os = "macos")]
extern crate ssh2;

#[cfg(feature = "sessionfs")]
extern crate fuse_mt;

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
    static ref STORAGE_DIR: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
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
    static ref UNIQUE_CLIENT_ID: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref TOKEN: ::parking_lot::RwLock<Token> = ::parking_lot::RwLock::new(Token::default());
}

lazy_static! {
    static ref CURRENT_USER: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref CANCEL_LIST: ::parking_lot::RwLock<Vec<String>> = ::parking_lot::RwLock::new(Vec::new());
}

lazy_static! {
    static ref LOG: ::parking_lot::RwLock<Vec<String>> = ::parking_lot::RwLock::new(Vec::new());
}
