pub static DEBUG_STATISTICS: bool = true;


pub static SDAPIDOMAIN_STAGING: &'static str = "staging.safedrive.io";
pub static SDAPIDOMAIN_PRODUCTION: &'static str = "safedrive.io";

pub static SDGROUP_NAME: &'static str = "group.io.safedrive.db";


pub static HMAC_SIZE: usize = ::sodiumoxide::crypto::auth::TAGBYTES;
pub static KEY_SIZE: usize = ::sodiumoxide::crypto::secretbox::KEYBYTES;
pub static NONCE_SIZE: usize = ::sodiumoxide::crypto::secretbox::NONCEBYTES;
pub static MAC_SIZE: usize = ::sodiumoxide::crypto::secretbox::MACBYTES;
