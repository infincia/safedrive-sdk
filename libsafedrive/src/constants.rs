use ::CONFIGURATION;

#[derive(Debug)]
pub enum Configuration {
    Staging,
    Production
}


pub static DEBUG_STATISTICS: bool = true;

static SD_WEB_DOMAIN_PRODUCTION: &'static str = "safedrive.io";
static SD_WEB_DOMAIN_STAGING: &'static str = "staging.safedrive.io";

static SD_API_DOMAIN_PRODUCTION: &'static str = "safedrive.io";
static SD_API_DOMAIN_STAGING: &'static str = "staging.safedrive.io";

pub static SDGROUP_NAME: &'static str = "group.io.safedrive.db";

pub static HMAC_KEY_SIZE: usize = ::sodiumoxide::crypto::auth::KEYBYTES;
pub static HMAC_SIZE: usize = ::sodiumoxide::crypto::auth::TAGBYTES;

pub static SECRETBOX_KEY_SIZE: usize = ::sodiumoxide::crypto::secretbox::KEYBYTES;
pub static SECRETBOX_NONCE_SIZE: usize = ::sodiumoxide::crypto::secretbox::NONCEBYTES;
pub static SECRETBOX_MAC_SIZE: usize = ::sodiumoxide::crypto::secretbox::MACBYTES;


pub fn is_production() -> bool {
    let c = CONFIGURATION.read().unwrap();
    match *c {
        Configuration::Staging => false,
        Configuration::Production => true,
    }
}

pub fn web_domain() -> &'static str {
    if is_production() {
        SD_WEB_DOMAIN_PRODUCTION
    } else {
        SD_WEB_DOMAIN_STAGING
    }
}

pub fn api_domain() -> &'static str {
    if is_production() {
        SD_API_DOMAIN_PRODUCTION
    } else {
        SD_API_DOMAIN_STAGING
    }
}
