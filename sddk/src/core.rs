use std::str;
use std::path::{Path, PathBuf};
use std::fs;
use std::ffi::CStr;

/// external crate imports
use simplelog::{Config as LogConfig, CombinedLogger, TermLogger, WriteLogger, SharedLogger};
use log::LogLevelFilter;

/// internal imports

use models::*;
use constants::*;
use sdapi::*;
use keys::*;

#[cfg(feature = "locking")]
use lock::FolderLock;

use error::{CryptoError, SDError};
use CONFIGURATION;
use CACHE_DIR;
use STORAGE_DIR;
use CLIENT_VERSION;
use USER_AGENT;
use OPERATING_SYSTEM;
use LANGUAGE_CODE;
use CHANNEL;
use LOG;
use CURRENT_USER;
use UNIQUE_CLIENT_ID;
use TOKEN;

use session::{SyncSession};

use remotefs::RemoteFS;

/// crypto exports

pub fn sha256(input: &[u8]) -> String {
    ::util::sha256(input)
}

/// helpers

pub use util::pretty_bytes;

pub use util::generate_uuid as generate_unique_client_id;
pub use util::get_current_os;

pub use cache::clean_cache;
pub use cache::clear_cache;

pub fn get_keychain_item(account: &str, service: ::keychain::KeychainService) -> Result<String, SDError> {
    let password = ::keychain::get_keychain_item(account, service)?;

    Ok(password)
}

pub fn set_keychain_item(account: &str, service: ::keychain::KeychainService, secret: &str) -> Result<(), SDError> {
    ::keychain::set_keychain_item(account, service, secret)?;

    Ok(())
}

pub fn delete_keychain_item(account: &str, service: ::keychain::KeychainService) -> Result<(), SDError> {
    ::keychain::delete_keychain_item(account, service)?;

    Ok(())
}

pub fn get_channel() -> Channel {
    let c = CHANNEL.read();

    match *c {
        Channel::Stable => Channel::Stable,
        Channel::Beta => Channel::Beta,
        Channel::Nightly => Channel::Nightly,
    }
}

pub fn get_version() -> String {
    let version: &str = env!("CARGO_PKG_VERSION");

    version.to_owned()
}

pub fn get_app_directory(config: &Configuration) -> Result<PathBuf, String> {
    ::util::get_app_directory(config)
}

/// internal functions

pub fn initialize<'a>(client_version: &'a str, desktop: bool, operating_system: &'a str, language_code: &'a str, config: Configuration, log_level: LogLevelFilter, local_storage_path: &Path) -> Result<(), SDError> {
    if !::sodiumoxide::init() {
        panic!("sodium initialization failed, cannot continue");
    }

    let openssl_version = unsafe {
        let c_openssl_version = ::openssl_sys::SSLeay_version(::openssl_sys::SSLEAY_VERSION);

        CStr::from_ptr(c_openssl_version).to_str().unwrap()
    };


    let sodium_version = ::sodiumoxide::version::version_string();
    let sdk_version = get_version();

    let mut c = CONFIGURATION.write();
    *c = config;

    let mut cv = CLIENT_VERSION.write();
    *cv = client_version.to_string();

    let mut os = OPERATING_SYSTEM.write();
    *os = operating_system.to_string();

    let mut lc = LANGUAGE_CODE.write();
    *lc = language_code.to_string();

    let app_type = match desktop {
        true => "desktop".to_owned(),
        false => "cli".to_owned(),
    };


    let mut ua = USER_AGENT.write();
    *ua = format!("SafeDrive/{} ({}; {}) SafeDriveSDK/{} libsodium/{}", client_version, operating_system, app_type, sdk_version, sodium_version);

    if let Err(e) = fs::create_dir_all(local_storage_path) {
        warn!("failed to create local directories: {}", e);
        return Err(SDError::from(e));
    }

    let storage_s = match local_storage_path.to_str() {
        Some(s) => s.to_string(),
        None => return Err(SDError::UnicodeError),
    };

    let mut sd = STORAGE_DIR.write();
    *sd = storage_s;



    let mut p = PathBuf::from(local_storage_path);
    p.push("cache");

    let cache_s = match p.as_path().to_str() {
        Some(s) => s.to_string(),
        None => return Err(SDError::UnicodeError),
    };
    if let Err(e) = fs::create_dir_all(&cache_s) {
        warn!("failed to create cache directories: {}", e);
        return Err(SDError::from(e));
    }
    let mut cd = CACHE_DIR.write();
    *cd = cache_s;

    let mut log_path = PathBuf::from(local_storage_path);
    let log_name = format!("safedrive-{}.log", app_type);

    log_path.push(&log_name);

    match ::std::fs::symlink_metadata(&log_path) {
        Ok(md) => {
            let stream_length = md.len();
            let is_file = md.file_type().is_file();
            if is_file && stream_length > 10_000_000 {
                let now = ::chrono::Utc::now();
                let log_backup_name = format!("safedrive-{}-{}.log", app_type, now);

                match ::std::fs::rename(&log_name, &log_backup_name) {
                    Ok(()) | Err(_) => {
                        // we're actually renaming the log file here, before logging is initialized
                        // so it's not as though we can log the error :D
                        // luckily this is going to be incredibly rare unless the drive is out of
                        // space, which we could add a check for
                    },
                }
            }
        },
        Err(_) => {},
    };

    let f = match ::std::fs::OpenOptions::new().read(true).append(true).create(true).open(&log_path) {
        Ok(file) => file,
        Err(e) => {
            return Err(SDError::Internal(format!("failed to open log file: {}", e)));
        },
    };
    let mut logs: Vec<Box<SharedLogger>> = Vec::new();

    let wl = WriteLogger::new(log_level, LogConfig::default(), f);
    logs.push(wl);

    match TermLogger::new(log_level, LogConfig::default()) {
        Some(logger) => {
            logs.push(logger);
        },
        None => {},
    }

    let sdl = ::sdlog::SDLogger::new(log_level, LogConfig::default());
    logs.push(sdl);

    let _ = match CombinedLogger::init(logs) {
        Ok(()) => {},
        Err(e) => {
            return Err(SDError::Internal(format!("failed to initialize log: {}", e)));
        },
    };


    info!("libsodium {}", sodium_version);


    info!("{}", openssl_version);

    info!("sddk ready");

    Ok(())
}

pub fn login(unique_client_id: &str,
             username: &str,
             password:  &str) -> Result<(Token, AccountStatus), SDError> {

    let gos = OPERATING_SYSTEM.read();
    let lc = LANGUAGE_CODE.read();

    match register_client(&**gos, &**lc, unique_client_id, username, password) {
        Ok(t) => {
            let mut cu = CURRENT_USER.write();
            *cu = username.to_string();

            let mut ucid = UNIQUE_CLIENT_ID.write();
            *ucid = unique_client_id.to_string();

            let mut token = TOKEN.write();
            *token = t.clone();

            match account_status(&t) {
                Ok(s) => Ok((t, s)),
                Err(e) => Err(SDError::from(e)),
            }
        },
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn start_daemon() -> Result<(), SDError> {


    Ok(())
}

pub fn remove_software_client(token: &Token) -> Result<(), SDError> {
    match unregister_client(token) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn get_software_clients(username: &str,
                            password:  &str) -> Result<Vec<SoftwareClient>, SDError> {
    match list_clients(username, password) {
        Ok(clients) => Ok(clients),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn get_account_status(token: &Token) -> Result<AccountStatus, SDError> {
    match account_status(token) {
        Ok(s) => Ok(s),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn get_account_details(token: &Token) -> Result<AccountDetails, SDError> {
    match account_details(token) {
        Ok(d) => Ok(d),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn load_keys(token: &Token, recovery_phrase: Option<String>, store_recovery_key: &Fn(&str), issue: &Fn(&str)) -> Result<Keyset, SDError> {
    /// generate new keys in all cases, the account *may* already have some stored, we only
    /// find out for sure while trying to store them.
    ///
    /// we do this on purpose:
    ///
    ///    1. to eliminate possible race conditions where two clients generate valid keysets and try
    ///       to store them at the same time, only the server knows who won and we only need one HTTP
    ///       request to find out
    ///
    ///    2. clients are never allowed to supply their own recovery phrases and keys to the SDK.
    ///       either we actually need new keys, in which case the ones we safely generated internally
    ///       are stored on the server immediately, or we're wrong and the server tells us what
    ///       to use instead
    ///
    ///
    /// in all cases, the library will only use keys that have been returned by the server, this
    /// guarantees that the keys we actually use are correct as long as the server is correct
    ///
    let new_wrapped_keyset = match WrappedKeyset::new() {
        Ok(wks) => wks,
        Err(_) => return Err(SDError::from(CryptoError::KeyGenerationFailed)),
    };


    match account_key(token, &new_wrapped_keyset) {
        Ok(real_wrapped_keyset) => {
            /// now we check to see if the keys returned by the server match the existing phrase or not

            /// if we were given an existing phrase try it, otherwise try the new one
            debug!("got key set back from server, checking");

            if let Some(p) = recovery_phrase {
                match real_wrapped_keyset.to_keyset(&p) {
                    Ok(ks) => {
                        if !ks.has_ecc {
                            issue("Warning: keys are legacy key type")
                        }
                        Ok(ks)
                    },
                    Err(e) => {
                        warn!("failed to decrypt keys: {}", e);
                        Err(SDError::from(e))
                    },
                }
            } else if let Some(p) = new_wrapped_keyset.recovery_phrase() {
                match real_wrapped_keyset.to_keyset(&p) {
                    Ok(ks) => {
                        if !ks.has_ecc {
                            issue("Warning: keys are legacy key type")
                        }

                        /// a new keyset was generated so we must return the phrase to the caller so it
                        /// can be stored and displayed
                        store_recovery_key(&p);
                        Ok(ks)
                    },
                    Err(e) => {
                        warn!("failed to decrypt keys: {}", e);
                        Err(SDError::from(e))
                    },
                }
            } else {
                unreachable!("");
            }
        },
        Err(e) => Err(SDError::from(e)),
    }
}

#[allow(unused_variables)]
pub fn get_sync_folder(token: &Token,
                       folder_id: u64) -> Result<RegisteredFolder, SDError> {
    let folders = match read_folders(token) {
        Ok(folders) => folders,
        Err(e) => return Err(SDError::from(e)),
    };
    for folder in folders {
        if folder.id == folder_id {
            return Ok(folder);
        }
    }
    Err(SDError::Internal(format!("unexpected failure to find folder_id {}", folder_id)))
}

#[allow(unused_variables)]
pub fn has_conflicting_folder(token: &Token,
                              folder_path: &Path) -> Result<bool, SDError> {
    let folders = match read_folders(token) {
        Ok(folders) => folders,
        Err(e) => return Err(SDError::from(e)),
    };
    if let Some(test_path) = folder_path.to_str() {
        let test_lower: String = test_path.to_lowercase();


        for folder in folders {
            // let options: NSString.CompareOptions = [.anchored, .caseInsensitive]

            // check if folder_path is a parent or subdirectory of an existing folder
            let folder_lower: String = folder.folderPath.as_str().to_lowercase();

            if test_lower.as_str().contains(folder_lower.as_str()) {
                return Ok(true);

            }
            if folder_lower.as_str().contains(test_lower.as_str()) {
                return Ok(true);

            }
        }
    }
    return Ok(false);
}

pub fn add_sync_folder(token: &Token,
                       name: &str,
                       path: &str,
                       encrypted: bool) -> Result<u64, SDError> {
    match create_folder(token, path, name, encrypted) {
        Ok(folder_id) => Ok(folder_id),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn update_sync_folder(token: &Token,
                          name: &str,
                          path: &str,
                          syncing: bool,
                          folder_id: u64) -> Result<(), SDError> {
    match update_folder(token, path, name, syncing, folder_id) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn remove_sync_folder(token: &Token,
                          folder_id: u64) -> Result<(), SDError> {
    match delete_folder(token, folder_id) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn get_sync_folders(token: &Token) -> Result<Vec<RegisteredFolder>, SDError> {
    match read_folders(token) {
        Ok(folders) => Ok(folders),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn get_sync_session<'a>(token: &Token,
                            folder_id: u64,
                            session: &'a str) -> Result<SyncSessionResponse<'a>, SDError> {
    match read_session(token, folder_id, session, true) {
        Ok(session) => Ok(session),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn get_sync_sessions(token: &Token) -> Result<Vec<SyncSession>, SDError> {
    let res = match read_sessions(token) {
        Ok(res) => res,
        Err(e) => return Err(SDError::from(e)),
    };
    let s = match res.get("sessionDetails") {
        Some(s) => s,
        None => return Err(SDError::Internal("failed to get sessionDetails from sessions response".to_string())),
    };

    let mut v: Vec<SyncSession> = Vec::new();

    for (folder_id, sessions) in s {
        for session in sessions {
            let mut ses = session.clone();
            ses.folder_id = Some(*folder_id);
            v.push(ses);
        }
    }

    Ok(v)
}

pub fn remove_sync_session(token: &Token,
                           session_id: u64) -> Result<(), SDError> {
    match delete_session(token, session_id) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn clean_sync_sessions(token: &Token, schedule: SyncCleaningSchedule) -> Result<(), SDError> {
    use ::chrono::{Local, Utc, Timelike};

    let utc_time = Utc::now();
    let local_time = utc_time.with_timezone(&Local);
    let midnight = local_time.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();

    match schedule {
        SyncCleaningSchedule::Auto => {

            Err(SDError::Internal("not implemented".to_string()))

        },
        SyncCleaningSchedule::ExactDateRFC3339 { date } => {
            let date = match ::chrono::DateTime::parse_from_rfc3339(&date) {
                Ok(dt) => dt,
                Err(e) => {
                    return Err(SDError::Internal(format!("{}", e)));
                },
            };

            remove_sync_sessions_before(token, ::util::timestamp_to_ms(date.timestamp(), date.timestamp_subsec_millis()))

        },
        SyncCleaningSchedule::ExactDateRFC2822 { date } => {
            let date = match ::chrono::DateTime::parse_from_rfc2822(&date) {
                Ok(dt) => dt,
                Err(e) => {
                    return Err(SDError::Internal(format!("{}", e)));
                },
            };

            remove_sync_sessions_before(token, ::util::timestamp_to_ms(date.timestamp(), date.timestamp_subsec_millis()))

        },
        SyncCleaningSchedule::All => {

            remove_sync_sessions_before(token, ::util::timestamp_to_ms(local_time.timestamp(), local_time.timestamp_subsec_millis()))

        },
        SyncCleaningSchedule::BeforeToday => {

            remove_sync_sessions_before(token, ::util::timestamp_to_ms(midnight.timestamp(), midnight.timestamp_subsec_millis()))

        },
        SyncCleaningSchedule::BeforeThisWeek => {

            Err(SDError::Internal("not implemented".to_string()))

        },
        SyncCleaningSchedule::BeforeThisMonth => {

            Err(SDError::Internal("not implemented".to_string()))

        },
        SyncCleaningSchedule::BeforeThisYear => {

            Err(SDError::Internal("not implemented".to_string()))

        },
        SyncCleaningSchedule::OneDay => {

            Err(SDError::Internal("not implemented".to_string()))

        },
        SyncCleaningSchedule::OneWeek => {

            Err(SDError::Internal("not implemented".to_string()))

        },
        SyncCleaningSchedule::OneMonth => {

            Err(SDError::Internal("not implemented".to_string()))

        },
        SyncCleaningSchedule::OneYear => {

            Err(SDError::Internal("not implemented".to_string()))

        },
    }
}

pub fn remove_sync_sessions_before(token: &Token,
                                   timestamp: i64) -> Result<(), SDError> {
    match delete_sessions(token, timestamp) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn log(text: &str, level: LogLevelFilter) {
    match level {
        ::log::LogLevelFilter::Off => {},
        ::log::LogLevelFilter::Error => {
            error!("{}", text);
        },
        ::log::LogLevelFilter::Warn => {
            warn!("{}", text);
        },
        ::log::LogLevelFilter::Info => {
            info!("{}", text);
        },
        ::log::LogLevelFilter::Debug => {
            debug!("{}", text);
        },
        ::log::LogLevelFilter::Trace => {
            trace!("{}", text);
        },
    };

}

pub fn send_error_report<'a>(client_version: Option<String>, operating_system: Option<String>, unique_client_id: &str, description: &str, context: &str) -> Result<(), SDError> {
    let gcv = CLIENT_VERSION.read();
    let gos = OPERATING_SYSTEM.read();

    // clone the memory log to avoid holding the lock for the duration of the API call
    let log_messages: Vec<String> = {
        let log = LOG.read();

        (*log).clone()
    };

    let cv: &str = match client_version {
        Some(ref cv) => cv,
        None => &**gcv,
    };

    let os: &str = match operating_system {
        Some(ref os) => os,
        None => &**gos,
    };

    match report_error(cv, os, unique_client_id, description, context, &log_messages) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn remote_mkdir(host: &str, port: u16, remote_path: &Path, username: &str, password: &str) -> Result<(), SDError> {
    let mut remote_fs = RemoteFS::new(host, port, username, password);

    match remote_fs.mkdir(true, remote_path) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn remote_rmdir(host: &str, port: u16, remote_path: &Path, username: &str, password: &str) -> Result<(), SDError> {
    let mut remote_fs = RemoteFS::new(host, port, username, password);

    match remote_fs.rmdir(remote_path) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn remote_rm(host: &str, port: u16, remote_path: &Path, recursive: bool, username: &str, password: &str) -> Result<(), SDError> {
    let mut remote_fs = RemoteFS::new(host, port, username, password);

    match remote_fs.rm(remote_path, recursive) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}

pub fn remote_mv(host: &str, port: u16, remote_path: &Path, new_path: &Path, username: &str, password: &str) -> Result<(), SDError> {
    let mut remote_fs = RemoteFS::new(host, port, username, password);

    match remote_fs.mv(remote_path, new_path) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e)),
    }
}


