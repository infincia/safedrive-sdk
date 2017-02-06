#[macro_use] extern crate log;

pub mod c_api;
pub use c_api::*;

pub mod core;
pub mod models;
pub mod error;
pub mod keys;

#[cfg(feature = "keychain")]
pub mod keychain;

pub mod constants;

mod util;
mod sdapi;
mod state;
mod binformat;
mod cache;
mod block;
mod session;
mod lock;

// external crates

extern crate rustc_serialize;
extern crate libc;
extern crate sodiumoxide;
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

#[cfg(target_os = "macos")]
extern crate interfaces;

#[cfg(target_os = "linux")]
extern crate openssl;

#[cfg(feature = "keychain")]
#[cfg(feature = "linux-keychain")]
#[cfg(target_os = "linux")]
extern crate secret_service;

#[cfg(target_os = "macos")]
extern crate objc_foundation;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[cfg(target_os = "macos")]
#[cfg(feature = "keychain")]
#[cfg(feature = "mac-keychain")]
extern crate security_framework;

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

pub static SYNC_VERSION: ::models::SyncVersion = ::models::SyncVersion::Version1;

// global config, can only be set once at runtime

lazy_static! {
    static ref CONFIGURATION: ::parking_lot::RwLock<constants::Configuration> = ::parking_lot::RwLock::new(constants::Configuration::Production);
}

lazy_static! {
    static ref CACHE_DIR: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref CLIENT_VERSION: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref OPERATING_SYSTEM: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}

lazy_static! {
    static ref LANGUAGE_CODE: ::parking_lot::RwLock<::std::string::String> = ::parking_lot::RwLock::new(::std::string::String::new());
}








