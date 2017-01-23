#[macro_use] extern crate log;

pub mod c_api;
pub use c_api::*;

pub mod core;
pub mod models;
pub mod error;
pub mod keys;

mod constants;
mod util;
mod sdapi;
mod state;
mod binformat;
mod cache;
mod session;

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

#[cfg(unix)]
extern crate interfaces;

#[cfg(target_os = "linux")]
extern crate openssl;

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

// global config, can only be set once at runtime

lazy_static! {
    static ref CONFIGURATION: std::sync::RwLock<models::Configuration> = std::sync::RwLock::new(models::Configuration::Production);
}

lazy_static! {
    static ref CACHE_DIR: std::sync::RwLock<::std::string::String> = std::sync::RwLock::new(::std::string::String::new());
}







