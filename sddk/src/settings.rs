use std::path::Path;
use std::io;
use winreg::RegKey;
use winreg::enums::*;

use error::SDError;

static SD_REG_PATH: &str = "Software\\SafeNet\\SafeDrive";

static SD_REG_KEY_DRIVE: &str = "Drive";
static SD_REG_KEY_EMAIL: &str = "Email";
static SD_REG_KEY_PASSWORD: &str = "Password";
static SD_REG_KEY_LANGUAGE: &str = "Language";
static SD_REG_KEY_SYNC_FOLDERS: &str = "SyncFolders";
static SD_REG_KEY_HOST: &str = "RegisterHost";
static SD_REG_KEY_HOST_PORT: &str = "RegisterHostPort";
static SD_REG_KEY_DEVICE: &str = "Device";
static SD_REG_KEY_CLIENT_ID: &str = "ClientID";

static SD_REG_STARTUP_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
static SD_REG_KEY_APP_NAME: &str = "SafeDrive";

pub fn get_key(key: &str) -> Result<String, SDError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let main = hkcu.open_subkey_with_flags(SD_REG_STARTUP_PATH, KEY_READ)?;
    let value: String = main.get_value(key)?;
    Ok(value)
}

pub fn set_key(key: &str, value: &str) -> Result<(), SDError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let main = hkcu.open_subkey_with_flags(SD_REG_STARTUP_PATH, KEY_READ)?;
    main.set_value(key, &value)?;

    Ok(())
}

pub fn delete_key(key: &str) -> Result<(), SDError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let main = hkcu.open_subkey_with_flags(SD_REG_STARTUP_PATH, KEY_READ)?;
    main.delete_value(key)?;
    Ok(())
}