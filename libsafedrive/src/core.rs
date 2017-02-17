use std::str;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read, Write, Seek, SeekFrom};
use std::{thread, time};
/// external crate imports

use ::rustc_serialize::hex::{ToHex, FromHex};
use ::rand::distributions::{IndependentSample, Range};
use ::tar::{Builder, Header, Archive, EntryType};
use ::walkdir::WalkDir;
use ::nom::IResult::*;

/// internal imports

use ::models::*;
use ::block::*;
use ::constants::*;
use ::sdapi::*;
use ::keys::*;

#[cfg(feature = "locking")]
use ::lock::{FolderLock};

use ::error::{CryptoError, SDAPIError, SDError};
use ::CONFIGURATION;
use ::CACHE_DIR;
use ::CLIENT_VERSION;
use ::USER_AGENT;
use ::OPERATING_SYSTEM;
use ::LANGUAGE_CODE;
use ::SYNC_VERSION;
use ::CHANNEL;

use ::session::{SyncSession, WrappedSyncSession};

/// crypto exports

pub fn sha256(input: &[u8]) -> String {
    ::util::sha256(input)
}

/// helpers

pub use ::util::pretty_bytes;

pub use ::util::get_app_directory;
pub use ::util::generate_uuid as generate_unique_client_id;
pub use ::util::get_unique_client_id;
pub use ::util::set_unique_client_id;
pub use ::util::get_current_os;
pub use ::util::get_openssl_directory;
#[cfg(target_os = "macos")]
pub use ::util::unique_client_hash;

pub use ::cache::clean_cache;
pub use ::cache::clear_cache;

#[cfg(feature = "keychain")]
pub fn get_keychain_item(account: &str, service: ::keychain::KeychainService) -> Result<String, SDError> {
    ::keychain::get_keychain_item(account, service)?
}

#[cfg(feature = "keychain")]
pub fn set_keychain_item(account: &str, service: ::keychain::KeychainService, secret: &str) -> Result<(), SDError> {
    ::keychain::set_keychain_item(account, service, secret)?
}

pub fn get_channel() -> Channel {
    let c = CHANNEL.read();

    match *c {
        Channel::Stable => {
            Channel::Stable
        },
        Channel::Beta => {
            Channel::Beta
        },
        Channel::Nightly => {
            Channel::Nightly
        },
    }
}



/// internal functions

pub fn initialize<'a>(client_version: &'a str, operating_system: &'a str, language_code: &'a str, config: Configuration, local_storage_path: &Path) -> Result<(), SDError> {
    let mut c = CONFIGURATION.write();
    *c = config;

    let mut cv = CLIENT_VERSION.write();
    *cv = client_version.to_string();

    let mut ua = USER_AGENT.write();
    *ua = str::replace(client_version, " ", "/").to_string();

    let mut os = OPERATING_SYSTEM.write();
    *os = operating_system.to_string();

    let mut lc = LANGUAGE_CODE.write();
    *lc = language_code.to_string();

    if let Err(e) = fs::create_dir_all(local_storage_path) {
        debug!("failed to create local directories: {}", e);
        return Err(SDError::from(e))
    }

    let mut p = PathBuf::from(local_storage_path);
    p.push("cache");

    let cache_s = match p.as_path().to_str() {
        Some(ref s) => s.to_string(),
        None => { return Err(SDError::UnicodeError) },
    };
    if let Err(e) = fs::create_dir_all(&cache_s) {
        debug!("failed to create local directories: {}", e);
        return Err(SDError::from(e))
    }
    let mut cd = CACHE_DIR.write();
    *cd = cache_s;

    if !::sodiumoxide::init() == true {
        panic!("sodium initialization failed, cannot continue");
    }

    let sodium_version = ::sodiumoxide::version::version_string();
    debug!("libsodium {}", sodium_version);

    #[cfg(target_os = "linux")]
    let ssl_version = ::openssl::version::version();
    #[cfg(target_os = "linux")]
    debug!("{}", ssl_version);

    debug!("ready");

    Ok(())
}

pub fn login(unique_client_id: &str,
             local_storage_path: &Path,
             username: &str,
             password:  &str) -> Result<(Token, AccountStatus), SDError> {




    match ::util::set_unique_client_id(unique_client_id, local_storage_path) {
        Ok(()) => {},
        Err(e) => {
            debug!("failed to set unique client id: {}", e);
            return Err(e)

        },
    }

    let gos = OPERATING_SYSTEM.read();
    let lc = LANGUAGE_CODE.read();

    match register_client(&**gos, &**lc, unique_client_id, username, password) {
        Ok(t) => {
            match account_status(&t) {
                Ok(s) => return Ok((t, s)),
                Err(e) => Err(SDError::from(e))
            }
        },
        Err(e) => return Err(SDError::from(e))
    }
}

pub fn get_account_status(token: &Token) -> Result<AccountStatus, SDError> {
    match account_status(&token) {
        Ok(s) => return Ok(s),
        Err(e) => Err(SDError::from(e))
    }
}

pub fn get_account_details(token: &Token) -> Result<AccountDetails, SDError> {
    match account_details(&token) {
        Ok(d) => return Ok(d),
        Err(e) => Err(SDError::from(e))
    }
}

pub fn load_keys(token: &Token, recovery_phrase: Option<String>, store_recovery_key: &Fn(&str)) -> Result<Keyset, SDError> {
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
        Err(_) => return Err(SDError::from(CryptoError::KeyGenerationFailed))
    };


    match account_key(token, &new_wrapped_keyset) {
        Ok(real_wrapped_keyset) => {
            /// now we check to see if the keys returned by the server match the existing phrase or not

            /// if we were given an existing phrase try it, otherwise try the new one
            debug!("got key set back from server, checking");

            if let Some(p) = recovery_phrase {
                match real_wrapped_keyset.to_keyset(&p) {
                    Ok(ks) => {
                        Ok(ks)
                    },
                    Err(e) => {
                        debug!("failed to decrypt keys: {}", e);
                        Err(SDError::RecoveryPhraseIncorrect)
                    }
                }
            } else if let Some(p) = new_wrapped_keyset.recovery_phrase() {
                match real_wrapped_keyset.to_keyset(&p) {
                    Ok(ks) => {
                        /// a new keyset was generated so we must return the phrase to the caller so it
                        /// can be stored and displayed
                        store_recovery_key(&p);
                        Ok(ks)
                    },
                    Err(e) => {
                        debug!("failed to decrypt keys: {}", e);
                        Err(SDError::from(e))
                    }
                }
            } else {
                unreachable!("");
            }
        },
        Err(e) => Err(SDError::from(e))
    }
}

#[allow(unused_variables)]
pub fn get_sync_folder(token: &Token,
                       folder_id: u64) -> Result<RegisteredFolder, SDError> {
    let folders = match read_folders(token) {
        Ok(folders) => folders,
        Err(e) => return Err(SDError::from(e))
    };
    for folder in folders {
        if folder.id == folder_id {
            return Ok(folder)
        }
    }
    return Err(SDError::Internal(format!("unexpected failure to find folder_id {}", folder_id)))
}

pub fn add_sync_folder(token: &Token,
                       name: &str,
                       path: &str) -> Result<u64, SDError> {
    match create_folder(token, path, name, true) {
        Ok(folder_id) => Ok(folder_id),
        Err(e) => Err(SDError::from(e))
    }
}

pub fn remove_sync_folder(token: &Token,
                          folder_id: u64) -> Result<(), SDError> {
    match delete_folder(token, folder_id) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e))
    }
}

pub fn get_sync_folders(token: &Token) -> Result<Vec<RegisteredFolder>, SDError> {
    match read_folders(token) {
        Ok(folders) => Ok(folders),
        Err(e) => Err(SDError::from(e))
    }
}

pub fn get_sync_session<'a>(token: &Token,
                            folder_id: u64,
                            session: &'a str) -> Result<SyncSessionResponse<'a>, SDError> {
    let session = match read_session(token, folder_id, session, true) {
        Ok(session) => session,
        Err(e) => return Err(SDError::from(e))
    };

    Ok(session)
}

pub fn get_sync_sessions(token: &Token) -> Result<Vec<SyncSession>, SDError> {
    let res = match read_sessions(token) {
        Ok(res) => res,
        Err(e) => return Err(SDError::from(e))
    };
    let s = match res.get("sessionDetails") {
        Some(s) => s,
        None => return Err(SDError::Internal(format!("failed to get sessionDetails from sessions response")))
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
        Err(e) => Err(SDError::from(e))
    }
}

pub fn clean_sync_sessions(token: &Token, schedule: SyncCleaningSchedule) -> Result<(), SDError> {
    use ::chrono::{Local, UTC, Timelike};

    let utc_time = UTC::now();
    let local_time = utc_time.with_timezone(&Local);
    let midnight = local_time.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();

    match schedule {
        SyncCleaningSchedule::Auto => {

            return Err(SDError::Internal("not implemented".to_string()));

        },
        SyncCleaningSchedule::ExactDateRFC3339 { date } => {
            let date = match ::chrono::DateTime::parse_from_rfc3339(&date) {
                Ok(dt) => dt,
                Err(e) => {
                    return Err(SDError::Internal(format!("{}", e)));
                }
            };

            remove_sync_sessions_before(token, ::util::timestamp_to_ms(date.timestamp(), date.timestamp_subsec_millis()))

        },
        SyncCleaningSchedule::ExactDateRFC2822 { date } => {
            let date = match ::chrono::DateTime::parse_from_rfc2822(&date) {
                Ok(dt) => dt,
                Err(e) => {
                    return Err(SDError::Internal(format!("{}", e)));
                }
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

            return Err(SDError::Internal("not implemented".to_string()));

        },
        SyncCleaningSchedule::BeforeThisMonth => {

            return Err(SDError::Internal("not implemented".to_string()));

        },
        SyncCleaningSchedule::BeforeThisYear => {

            return Err(SDError::Internal("not implemented".to_string()));

        },
        SyncCleaningSchedule::OneDay => {

            return Err(SDError::Internal("not implemented".to_string()));

        },
        SyncCleaningSchedule::OneWeek => {

            return Err(SDError::Internal("not implemented".to_string()));

        },
        SyncCleaningSchedule::OneMonth => {

            return Err(SDError::Internal("not implemented".to_string()));

        },
        SyncCleaningSchedule::OneYear => {

            return Err(SDError::Internal("not implemented".to_string()));

        },
    }
}

pub fn remove_sync_sessions_before(token: &Token,
                                   timestamp: i64) -> Result<(), SDError> {
    match delete_sessions(token, timestamp) {
        Ok(()) => Ok(()),
        Err(e) => Err(SDError::from(e))
    }
}

pub fn sync(token: &Token,
            session_name: &str,
            main_key: &Key,
            hmac_key: &Key,
            tweak_key: &Key,
            folder_id: u64,
            progress: &mut FnMut(u64, u64, u64, f64, bool, &str)) -> Result<(), SDError> {
    debug!("creating version {} sync session", SYNC_VERSION);

    let folder = match get_sync_folder(token, folder_id) {
        Ok(folder) => folder,
        Err(e) => return Err(SDError::from(e))
    };

    let folder_path = PathBuf::from(&folder.folderPath);
    let folder_name = &folder.folderName;

    #[cfg(feature = "locking")]
    let flock = try!(FolderLock::new(&folder_path));

    #[cfg(feature = "locking")]
    defer!({
        /// try to ensure the folder goes back to unlocked state, but if not there isn't anything
        /// we can do about it
        flock.unlock();
    });

    match register_sync_session(token, folder_id, session_name, true) {
        Ok(()) => {},
        Err(e) => return Err(SDError::from(e))
    };



    let archive_file = Vec::new();

    debug!("creating session for: {} (folder id {})", folder_name, folder_id);


    let mut ar = Builder::new(archive_file);

    let mut processed_size: u64 = 0;
    let mut processed_size_compressed: u64 = 0;
    let mut processed_size_padding: u64 = 0;

    let mut estimated_size: u64 = 0;

    for item in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
        let p = item.path();

        let md = match ::std::fs::symlink_metadata(&p) {
            Ok(m) => m,
            Err(_) => { continue },
        };

        let stream_length = md.len();
        debug!("estimating size of {}... OK, {}", item.path().display(), stream_length);

        estimated_size = estimated_size + stream_length;
    }

    let mut failed = 0;

    let mut percent_completed: f64 = (processed_size as f64 / estimated_size as f64) * 100.0;

    for item in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
        debug!("examining {}", item.path().display());

        percent_completed = (processed_size as f64 / estimated_size as f64) * 100.0;

        /// call out to the library user with progress
        progress(estimated_size, processed_size, 0, percent_completed, false, "");

        let p = item.path();
        let p_relative = p.strip_prefix(&folder_path).expect("failed to unwrap relative path");

        let md = match ::std::fs::symlink_metadata(&p) {
            Ok(m) => m,
            Err(_) => { failed = failed +1; continue },
        };

        let stream_length = md.len();
        let is_file = md.file_type().is_file();
        let is_dir = md.file_type().is_dir();
        let is_symlink = md.file_type().is_symlink();

        /// store metadata for directory or file
        let mut header = Header::new_gnu();
        if let Err(err) = header.set_path(p_relative) {
            debug!("not adding invalid path: '{}' (reason: {})", p_relative.display(), err);

            failed + failed + 1;
            continue // we don't care about errors here, they'll only happen for truly invalid paths
        }
        header.set_metadata(&md);

        let mut hmac_bag: Vec<u8> = Vec::new();

        // chunk file if not a directory or socket
        if is_file {
            if stream_length > 0 {

                let mut block_generator = ::chunk::BlockGenerator::new(&p, main_key, hmac_key, tweak_key, stream_length, SYNC_VERSION);

                let mut skipped_blocks = 0;

                let mut item_padding: u64 = 0;

                for block_result in block_generator.by_ref() {
                    let block = match block_result {
                        Ok(b) => b,
                        Err(e) => {
                            failed = failed +1; continue;
                        }
                    };

                    let block_name = block.name();
                    let block_real_size = block.real_size();
                    let compressed = block.compressed();

                    processed_size += block_real_size;

                    let block_compressed_size = match block.compressed_size() {
                        Some(size) => {
                            processed_size_compressed += size;

                            size
                        },
                        None => {
                            processed_size_compressed += block_real_size;

                            0
                        },
                    };

                    hmac_bag.extend_from_slice(&block.get_hmac());

                    let wrapped_block = match block.to_wrapped(main_key) {
                        Ok(wb) => wb,
                        Err(e) => return Err(SDError::CryptoError(e)),
                    };
                    let block_padded_size = wrapped_block.len() as u64;

                    let padding_overhead = if compressed {
                        block_padded_size - block_compressed_size
                    } else {
                        block_padded_size - block_real_size
                    };

                    item_padding += padding_overhead;

                    let mut should_retry = true;
                    let mut retries_left = 15.0;
                    let mut should_upload = false;

                    while should_retry {
                        /// allow caller to tick the progress display, if one exists
                        progress(estimated_size, processed_size, 0, percent_completed, false, "");

                        let failed_count = 15.0 - retries_left;
                        let mut rng = ::rand::thread_rng();

                        /// we pick a multiplier randomly to avoid a bunch of clients trying again
                        /// at the same 2/4/8/16 backoff times time over and over if the server
                        /// is overloaded or down
                        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

                        if failed_count >= 2.0 && retries_left > 0.0 {
                            /// back off significantly every time a call fails but only after the
                            /// second try, the first failure could be us not including the data
                            /// when we should have
                            let backoff_time = backoff_multiplier * (failed_count * failed_count);
                            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
                            thread::sleep(delay);
                        }

                        /// tell the server to mark this block without the data first, if that fails we try uploading

                        match write_block(&token, session_name, &block_name, &wrapped_block, should_upload) {
                            Ok(()) => {
                                skipped_blocks = skipped_blocks + 1;
                                /// allow caller to tick the progress display, if one exists
                                progress(estimated_size, processed_size, block_real_size as u64, percent_completed, false, "");
                                should_retry = false
                            },
                            Err(SDAPIError::RequestFailed(err)) => {
                                progress(estimated_size, processed_size, 0, percent_completed, false, err.description());

                                retries_left = retries_left - 1.0;
                                if retries_left <= 0.0 {
                                    /// TODO: pass better error info up the call chain here rather than a failure
                                    return Err(SDError::RequestFailure(err))
                                }
                            },
                            Err(SDAPIError::RetryUpload) => {
                                retries_left = retries_left - 1.0;
                                if retries_left <= 0.0 {
                                    return Err(SDError::ExceededRetries(15))
                                }
                                should_upload = true;
                            },
                            _ => {}
                        }
                    }
                }

                processed_size_padding += item_padding;


                let stats = block_generator.stats();
                if DEBUG_STATISTICS {
                    let compression_ratio = (stats.processed_size_compressed as f64 / stats.processed_size as f64 ) * 100.0;

                    debug!("{} chunks ({} new)", stats.discovered_chunk_count, stats.discovered_chunk_count - skipped_blocks);
                    debug!("average size: {} bytes", stats.processed_size / stats.discovered_chunk_count);
                    debug!("compression: {}/{} ({}%)", stats.processed_size_compressed, stats.processed_size, compression_ratio);

                    let padding_ratio = (item_padding as f64 / stats.processed_size_compressed as f64 ) * 100.0;

                    debug!("padding overhead: {} ({}%)", item_padding, padding_ratio);

                    debug!("hmac bag has: {} ids <{} bytes>", hmac_bag.len() / 32, stats.discovered_chunk_count * 32);
                    debug!("expected chunk size: {} bytes", stats.discovered_chunk_expected_size);
                    debug!("smallest chunk: {} bytes", stats.discovered_chunk_smallest_size);
                    debug!("largest chunk: {} bytes", stats.discovered_chunk_largest_size);
                    debug!("standard size deviation: {} bytes", (stats.discovered_chunk_size_variance as f64 / stats.discovered_chunk_count as f64).sqrt() as u64);
                }

                assert!(stats.processed_size == stream_length);
                debug!("calculated {} real bytes of blocks, matching stream size {}", stats.processed_size, stream_length);

                header.set_size(stats.discovered_chunk_count * HMAC_SIZE as u64); /// hmac list size
                header.set_cksum();
                ar.append(&header, hmac_bag.as_slice()).expect("failed to append session entry header");

            } else {
                header.set_size(0); /// hmac list size is zero when file has no actual data
                header.set_cksum();
                ar.append(&header, hmac_bag.as_slice()).expect("failed to append zero length archive header");
            }
        } else if is_dir {
            /// folder
            header.set_size(0); /// hmac list size is zero when file has no actual data
            header.set_cksum();
            ar.append(&header, hmac_bag.as_slice()).expect("failed to append folder to archive header");
        } else if is_symlink {
            /// symlink

            /// get the src
            match ::std::fs::read_link(&p) {
                Ok(path) => {
                    match  header.set_link_name(path) {
                        Ok(()) => {

                        },
                        Err(e) => {
                            error!("failed to set symlink: {}", e);
                        }
                    };
                },
                Err(e) => {
                    error!("failed to set symlink: {}", e);
                }
            };

            header.set_size(0); /// hmac list size is zero when file has no actual data
            header.set_cksum();
            ar.append(&header, hmac_bag.as_slice()).expect("failed to append symlink to archive header");
        }
    }

    /// since we're writing to a buffer in memory there shouldn't be any errors here...
    /// unless the system is also completely out of memory, but there's nothing we can do about that,
    /// so if it proves to be an issue we'll have to look for anything else that might use a lot of
    /// memory and use temp files instead, where possible
    let raw_session = ar.into_inner().unwrap();


    let session = SyncSession::new(SYNC_VERSION, folder_id, session_name.to_string(), Some(processed_size), None, raw_session);

    if DEBUG_STATISTICS {
        let compression_ratio = (processed_size_compressed as f64 / session.size.unwrap() as f64) * 100.0;
        debug!("session data total: {}", session.size.unwrap());
        debug!("session data compressed: {}", processed_size_compressed);
        debug!("session data compression ratio: {}", compression_ratio);
        let padding_ratio = (processed_size_padding as f64 / processed_size_compressed as f64 ) * 100.0;

        debug!("session data padding overhead: {} ({}%)", processed_size_padding, padding_ratio);

        debug!("session file total: {}", session.real_size());
        match session.compressed_size() {
            Some(size) => {
                let compression_ratio = (size as f64 / session.real_size() as f64) * 100.0;

                debug!("session file total compressed: {}", size);
                debug!("session file compression ratio: {}", compression_ratio);
            },
            None => {}
        }
    }


    let wrapped_session = match session.to_wrapped(main_key) {
        Ok(ws) => ws,
        Err(e) => return Err(SDError::CryptoError(e)),
    };

    let binary_data = wrapped_session.to_binary();


    debug!("finishing sync session");

    let mut should_retry = true;
    let mut retries_left = 15.0;

    while should_retry {
        /// allow caller to tick the progress display, if one exists
        progress(estimated_size, processed_size, 0, percent_completed, false, "");

        let failed_count = 15.0 - retries_left;
        let mut rng = ::rand::thread_rng();

        /// we pick a multiplier randomly to avoid a bunch of clients trying again
        /// at the same 2/4/8/16 backoff times time over and over if the server
        /// is overloaded or down
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1.0 {
            /// back off significantly every time a call fails
            let backoff_time = backoff_multiplier * (failed_count * failed_count);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        match finish_sync_session(&token, folder_id, session_name, true, &binary_data, processed_size as usize) {
            Ok(()) => {
                should_retry = false;
            },
            Err(e) => {
                retries_left = retries_left - 1.0;
                if retries_left <= 0.0 {
                    return Err(SDError::RequestFailure(Box::new(e)))
                }
            }
        };
    }


    progress(estimated_size, processed_size, 0, 100.0, false, "");

    Ok(())
}


pub fn restore(token: &Token,
               session_name: &str,
               main_key: &Key,
               folder_id: u64,
               destination: PathBuf,
               session_size: u64,
               progress: &mut FnMut(u64, u64, u64, f64, bool, &str)) -> Result<(), SDError> {
    try!(fs::create_dir_all(&destination));

    let folder = match get_sync_folder(token, folder_id) {
        Ok(folder) => folder,
        Err(e) => return Err(SDError::from(e))
    };

    let folder_name = &folder.folderName;

    #[cfg(feature = "locking")]
    let flock = try!(FolderLock::new(&destination));

    #[cfg(feature = "locking")]
    defer!({
        /// try to ensure the folder goes back to unlocked state, but if not there isn't anything
        /// we can do about it
        flock.unlock();
    });

    let session_body = match read_session(token, folder_id, session_name, true) {
        Ok(session_data) => session_data,
        Err(e) => return Err(SDError::from(e))
    };


    debug!("restoring session for: {} (folder id {})", folder_name, folder_id);



    let w_session = match WrappedSyncSession::from(session_body) {
        Ok(ws) => ws,
        Err(e) => return Err(e),
    };

    let session = match w_session.to_session(main_key) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let mut processed_size: u64 = 0;

    let mut ar = Archive::new(session.as_ref());

    let mut failed = 0;

    for item in ar.entries().unwrap() {
        let percent_completed: f64 = (processed_size as f64 / session_size as f64) * 100.0;


        let mut file_entry = match item {
            Ok(e) => e,
            Err(e) => {
                debug!("not restoring invalid session entry: {})", e);

                failed + failed + 1;
                continue // we do care about errors here, but we can't really recover from them for this item
            }
        };

        let mut full_p = PathBuf::from(&destination);

        match file_entry.path() {
            Ok(ref p) => {
                debug!("examining {}", &p.display());

                full_p.push(p);
            }
            Err(e) => {
                debug!("not restoring invalid path: {}", e);
                progress(session_size, processed_size, 0, percent_completed, false, &format!("not restoring invalid path: {}", e));

                failed + failed + 1;
                continue // we do care about errors here, but we can't really recover from them for this item
            }
        };


        /// call out to the library user with progress



        let entry_type = file_entry.header().entry_type();

        /// process if not a directory or socket
        match entry_type {
            EntryType::Regular => {
                let f = match File::create(&full_p) {
                    Ok(file) => file,
                    Err(err) => {
                        debug!("not able to create file at path: {})", err);
                        failed = failed +1;
                        continue
                    },
                };

                let stream_length = file_entry.header().size().unwrap();

                debug!("entry has {} blocks", stream_length / 32);
                
                if stream_length > 0 {
                    let mut stream = BufWriter::new(f);

                    let mut block_hmac_bag = Vec::new();

                    try!(file_entry.read_to_end(&mut block_hmac_bag));
                    let block_hmac_list = match ::binformat::parse_hmacs(&block_hmac_bag) {
                        Done(_, o) => o,
                        Error(e) => {
                            debug!("hmac bag parsing failed: {}", e);

                            return Err(SDError::CryptoError(CryptoError::SessionDecryptFailed))
                        },
                        Incomplete(_) => panic!("should never happen")
                    };


                    for block_hmac in block_hmac_list.iter() {

                        /// allow caller to tick the progress display, if one exists
                        progress(session_size, processed_size, 0, percent_completed, true, "");

                        let mut should_retry = true;
                        let mut retries_left = 15.0;

                        let block_hmac_hex = block_hmac.to_hex();
                        debug!("processing block {}", &block_hmac_hex);


                        let mut wrapped_block: Option<WrappedBlock> = None;

                        while should_retry {

                            let failed_count = 15.0 - retries_left;
                            let mut rng = ::rand::thread_rng();

                            /// we pick a multiplier randomly to avoid a bunch of clients trying again
                            /// at the same 2/4/8/16 back off time over and over if the server
                            /// is overloaded or down
                            let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

                            if failed_count >= 2.0 && retries_left > 0.0 {
                                /// back off significantly every time a call fails but only after the
                                /// second try, the first failure could be us not including the data
                                /// when we should have
                                let backoff_time = backoff_multiplier * (failed_count * failed_count);
                                let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
                                thread::sleep(delay);
                            }

                            /// get block from cache if possible
                            match ::cache::read_block(&block_hmac_hex) {
                                Ok(br) => {
                                    wrapped_block = Some(br);
                                    debug!("cache provided block: {}", &block_hmac_hex);
                                    break;
                                },
                                _ => {}
                            };

                            /// get block from the server

                            match ::sdapi::read_block(&token, &block_hmac_hex) {
                                Ok(rb) => {
                                    /// allow caller to tick the progress display, if one exists
                                    progress(session_size, processed_size, 0, percent_completed, false, "");

                                    should_retry = false;
                                    debug!("server provided block: {}", &block_hmac_hex);
                                    let wb = match WrappedBlock::from(rb, (&block_hmac_hex).from_hex().unwrap()) {
                                        Ok(wb) => wb,
                                        Err(e) => {
                                            debug!("block failed validation: {}", &block_hmac_hex);

                                            return Err(e)
                                        },
                                    };
                                    debug!("block processed: {}", &block_hmac_hex);

                                    match ::cache::write_block(&wb) {
                                        _ => {}
                                    };
                                    wrapped_block = Some(wb);
                                },
                                Err(SDAPIError::RequestFailed(err)) => {
                                    retries_left = retries_left - 1.0;

                                    progress(session_size, processed_size, 0, percent_completed, false, err.description());


                                    if retries_left <= 0.0 {
                                        /// TODO: pass better error info up the call chain here rather than a failure
                                        return Err(SDError::RequestFailure(err))
                                    }
                                },
                                _ => {}
                            };
                        }

                        let wrapped_block_s = wrapped_block.unwrap();

                        let block = match wrapped_block_s.to_block(main_key) {
                            Ok(b) => b,
                            Err(e) => return Err(e),
                        };
                        debug!("block unwrapped");

                        {
                            let new_position = stream.seek(SeekFrom::Current(0)).unwrap();
                            debug!("write start {:?}", new_position);

                        }
                        debug!("writing block segment of {} bytes", block.len());

                        {
                            try!(stream.write_all(block.as_ref()));
                        }
                        {
                            let new_position = stream.seek(SeekFrom::Current(0)).unwrap();
                            debug!("new position {:?}", new_position);

                        }

                        processed_size += block.len() as u64;
                        progress(session_size, processed_size, block.len() as u64, percent_completed, false, "");

                    }

                } else {
                    // empty file, just write one out with the same metadata but no body

                }
            },
            EntryType::Directory => {
                try!(fs::create_dir_all(&full_p));
            },
            EntryType::Link => {
                let src = match file_entry.link_name() {
                    Ok(op) => {
                        match op {
                            Some(pa) => pa,
                            None => {
                                debug!("not restoring invalid hard link: no link name found");
                                continue;
                            },
                        }
                    },
                    Err(e) => {
                        debug!("not restoring invalid hard link: {})", e);
                        continue;
                    }
                };


                match ::std::fs::hard_link(&src, full_p) {
                    Ok(()) => {},
                    Err(e) => {
                        error!("failed to restore hard link: {})", e);
                    },
                }

            },
            EntryType::Symlink => {
                let src = match file_entry.link_name() {
                    Ok(op) => {
                        match op {
                            Some(pa) => pa,
                            None => {
                                debug!("not restoring invalid symlink: no link name found");
                                continue;
                            },
                        }
                    },
                    Err(e) => {
                        debug!("not restoring invalid symlink: {})", e);
                        continue;
                    }
                };

                if src.iter().count() == 0 {
                    debug!("not restoring invalid symlink: destination not found");
                    continue;
                }

                #[cfg(unix)]
                match ::std::os::unix::fs::symlink(&src, full_p) {
                    Ok(()) => {},
                    Err(e) => {
                        error!("failed to restore symlink: {})", e);
                    },
                }

                #[cfg(windows)] {
                    let md = match ::std::fs::metadata(&src) {
                        Ok(m) => m,
                        Err(e) => {
                            debug!("not restoring symlink: {})", e);
                            continue
                        },
                    };

                    let is_dir = md.file_type().is_dir();
                    
                    if is_dir {
                        match ::std::os::windows::fs::symlink_dir(&src, full_p) {
                            Ok(()) => {},
                            Err(e) => {
                                error!("failed to restore directory symlink: {})", e);
                            },
                        }
                    } else {
                        match ::std::os::windows::fs::symlink_file(&src, full_p) {
                            Ok(()) => {},
                            Err(e) => {
                                error!("failed to restore file symlink: {})", e);
                            },
                        }
                    }
                }


            },
            EntryType::Char => {

            },
            EntryType::Block => {

            },
            EntryType::Fifo => {

            },
            EntryType::Continuous => {

            },
            EntryType::GNULongLink => {

            },
            EntryType::GNULongName => {

            },
            EntryType::GNUSparse => {

            },
            EntryType::XGlobalHeader => {

            },
            EntryType::XHeader => {

            },
            _ => {

            },
        }
    }

    debug!("restoring session finished");

    progress(session_size, processed_size, 0, 100.0, false, "");

    Ok(())
}

pub fn send_error_report<'a>(client_version: Option<String>, operating_system: Option<String>, unique_client_id: &str, description: &str, context: &str, log: &'a [&'a str]) -> Result<(), SDError> {
    let gcv = CLIENT_VERSION.read();
    let gos = OPERATING_SYSTEM.read();

    let cv: &str = match client_version {
        Some(ref cv) => cv,
        None => {
            &**gcv
        }
    };

    let os: &str = match operating_system {
        Some(ref os) => os,
        None => {
            &**gos
        }
    };

    match report_error(cv, os, unique_client_id, description, context, log) {
        Ok(()) => return Ok(()),
        Err(e) => Err(SDError::from(e))
    }
}