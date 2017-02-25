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
use ::binformat::BinaryWriter;

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
use ::CANCEL_LIST;

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

pub fn get_version() -> String {
    let version: &str = env!("CARGO_PKG_VERSION");

    version.to_owned()
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

pub fn cancel_sync_task(name: &str) {
    let mut cl = CANCEL_LIST.write();
    cl.push(name.to_owned());
}

pub fn is_sync_task_cancelled(name: String) -> bool {
    let mut cl = CANCEL_LIST.write();
    let mut cancelled = false;

    { if cl.contains(&name) { cancelled = true; } }

    { if cancelled { cl.retain(|&ref x| x != &name); }}

    cancelled
}

fn upload_thread(token: &Token, session_name: &str) -> (::std::thread::JoinHandle<()>, ::std::sync::mpsc::SyncSender<::cache::WriteCacheMessage<WrappedBlock>>, ::std::sync::mpsc::Receiver<Result<usize, SDError>>) {
    let item_limit = 300;
    let size_limit = 10_000_000;

    let mut write_cache: ::cache::WriteCache<WrappedBlock> = ::cache::WriteCache::new(item_limit, size_limit);

    let (block_send, block_receive) = ::std::sync::mpsc::sync_channel::<::cache::WriteCacheMessage<WrappedBlock>>(0);
    let (status_send, status_receive) = ::std::sync::mpsc::channel::<Result<usize, SDError>>();

    let local_token = token.clone();

    let local_session_name = session_name.to_owned();

    let write_thread: ::std::thread::JoinHandle<()> = thread::spawn(move || {

        loop {

            // check to see if we're finished, we need to ensure that we have already put the last
            // block in the cache and that the cache is now empty before exiting
            if write_cache.received_final_item() == true && write_cache.items_waiting() == 0 {
                return;
            }


            // if the cache is not full, receive a new message from the block iteration thread,
            // this should keep the cache close to full most of the time without stuttering

            if !write_cache.full() {
                debug!("write cache not full");

                let cache_message = match block_receive.recv() {
                    Ok(message) => message,
                    Err(e) => {
                        debug!("WriteCacheMessage: end of channel {}", e);
                        break;
                    },
                };

                // if we were told to stop processing blocks, just stop immediately
                if cache_message.stop {
                    return;
                }

                // check to see if we were given a new block, if so store it in the cache. if we don't
                // get a block here, it will be because the block iterator thread is completely done
                // and waiting for this thread to finish uploading them
                match cache_message.item {
                    Some(block) => {
                        write_cache.add(block);
                    },
                    None => {
                        write_cache.set_received_final_item(true);
                    }
                };
            } else {
                debug!("write cache full");
            }

            let pretty_size = ::util::pretty_bytes(write_cache.data_waiting() as f64);

            debug!("{} blocks waiting ({})", write_cache.items_waiting(), pretty_size);

            // The write cache has both size and item limits, because we need to avoid sending
            // too many blocks or too little data with each request. We try to achieve a balance,
            // sending more blocks if they are very small, fewer if they are very large.
            //
            // For example:
            //
            // A group of 100x 256KiB blocks is 25MiB
            // a group of 100x 256B blocks is 25KiB
            //
            // The tiny group is not worth the HTTP request unless we have no more waiting, but even
            // sending 5x more of the smaller blocks will still only get us a 1MiB request

            let block_batch = match write_cache.request_waiting_items() {
                Some(block_batch) => block_batch,
                None => {
                    debug!("not enough in cache to pop a group, waiting");
                    continue
                },
            };

            if block_batch.len() <= 0 {
                debug!("cache returned a zero length group, skipping request");

                continue; // failsafe to prevent sending requests with no blocks
            }

            debug!("sending {} blocks", block_batch.len());

            let block_write_start_time = ::std::time::Instant::now();



            let mut should_retry = true;
            let mut retries_left = 15.0;

            while should_retry {
                let failed_count = 15.0 - retries_left;
                let mut rng = ::rand::thread_rng();

                /// pick a random multiplier, avoid letting a bunch of clients retry the request
                /// at the same 2/4/8/16 backoff times to limit server load spikes

                let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

                if retries_left > 0.0 {
                    let backoff_time = backoff_multiplier * (failed_count * failed_count);
                    let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
                    debug!("backing off for {} seconds", delay.as_secs());

                    thread::sleep(delay);
                }

                match write_blocks(&local_token, &local_session_name, &block_batch) {
                    Ok(missing) => {
                        debug!("sending group took {} seconds", block_write_start_time.elapsed().as_secs());
                        should_retry = false;

                        let mut total_length = 0;
                        let mut missing_length = 0;

                        for block in &block_batch {
                            total_length += block.len();
                            let bn = &block.name();
                            if missing.contains(bn) {
                                debug!("server missing block, returning to the cache: {}", bn);
                                missing_length += block.len();

                                let mut c = block.clone();
                                c.needs_upload();
                                write_cache.add(c);
                            }
                        }


                        match status_send.send(Ok(total_length - missing_length)) {
                            Ok(()) => {},
                            Err(e) => {
                                debug!("channel exited: {}", e);
                                break;
                            },
                        }
                    },
                    Err(SDAPIError::RequestFailed(_)) => {
                        retries_left = retries_left - 1.0;
                        if retries_left <= 0.0 {
                            match status_send.send(Err(SDError::ExceededRetries(15))) {
                                Ok(()) => {},
                                Err(e) => {
                                    debug!("channel exited: {}", e);
                                    break;
                                },
                            }
                        }
                    },
                    Err(SDAPIError::Authentication) => {
                        match status_send.send(Err(SDError::Authentication)) {
                            Ok(()) => {},
                            Err(e) => {
                                debug!("channel exited: {}", e);
                                break;
                            },
                        }
                    },
                    _ => {}
                }

            }

        }
    });

    (write_thread, block_send, status_receive)
}

pub fn sync(token: &Token,
            session_name: &str,
            main_key: &Key,
            hmac_key: &Key,
            tweak_key: &Key,
            folder_id: u64,
            progress: &mut FnMut(u64, u64, u64, f64, bool),
            issue: &mut FnMut(&str)) -> Result<(), SDError> {
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
        let item_path = item.path();

        let md = match ::std::fs::symlink_metadata(&item_path) {
            Ok(m) => m,
            Err(e) => {
                issue(&format!("not able to sync file {}: {}", item_path.display(), e));
                continue
            },
        };

        let stream_length = md.len();
        trace!("estimating size of {}... OK, {}", item_path.display(), stream_length);

        estimated_size = estimated_size + stream_length;
    }

    let (write_thread, block_send, status_receive) = upload_thread(token, session_name);

    let mut failed = 0;

    let mut percent_completed: f64 = (processed_size as f64 / estimated_size as f64) * 100.0;

    for item in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
        if is_sync_task_cancelled(session_name.to_owned()) {
            let cache_message = ::cache::WriteCacheMessage::new(None, true, None);

            match block_send.send(cache_message) {
                Ok(()) => {

                },
                Err(e) => {
                    issue(&format!("not able to cancel upload: ({})", e));
                },
            }
            return Err(SDError::Cancelled)
        }

        debug!("examining {}", item.path().display());

        percent_completed = (processed_size as f64 / estimated_size as f64) * 100.0;

        /// call out to the library user with progress
        progress(estimated_size, processed_size, 0, percent_completed, false);

        let full_path = item.path();
        if &full_path == &folder_path {
            continue; // don't bother doing anything more for the root directory of the folder
        }
        let relative_path = full_path.strip_prefix(&folder_path).expect("failed to unwrap relative path");

        let md = match ::std::fs::symlink_metadata(&full_path) {
            Ok(m) => m,
            Err(_) => { failed = failed +1; continue },
        };

        let stream_length = md.len();
        let is_file = md.file_type().is_file();
        let is_dir = md.file_type().is_dir();
        let is_symlink = md.file_type().is_symlink();

        /// store metadata for directory or file
        let mut header = Header::new_gnu();
        if let Err(e) = header.set_path(relative_path) {
            issue(&format!("not able to sync file {}: {}", relative_path.display(), e));

            failed + failed + 1;
            continue // we don't care about errors here, they'll only happen for truly invalid paths
        }
        header.set_metadata(&md);

        let mut hmac_bag: Vec<u8> = Vec::new();

        // chunk file if not a directory or socket
        if is_file {
            if stream_length > 0 {

                let mut block_generator = ::chunk::BlockGenerator::new(&full_path, main_key, hmac_key, tweak_key, stream_length, SYNC_VERSION);

                let mut item_padding: u64 = 0;

                let mut block_failed = false;

                for block_result in block_generator.by_ref() {
                    if is_sync_task_cancelled(session_name.to_owned()) {
                        let cache_message = ::cache::WriteCacheMessage::new(None, true, None);

                        match block_send.send(cache_message) {
                            Ok(()) => {

                            },
                            Err(e) => {
                                issue(&format!("not able to cancel upload: ({})", e));
                            },
                        }
                        return Err(SDError::Cancelled)
                    }

                    match status_receive.try_recv() {
                        Ok(msg) => {
                            match msg {
                                Ok(sent) => {
                                    processed_size += sent as u64;

                                    progress(estimated_size, processed_size, sent as u64, percent_completed, false);
                                },
                                Err(e) => return Err(e)
                            };
                        },
                        Err(e) => {
                            match e {
                                ::std::sync::mpsc::TryRecvError::Empty => {},
                                ::std::sync::mpsc::TryRecvError::Disconnected => {
                                    debug!("Result<(), SDError>: end of channel {}", e);
                                    return Err(SDError::Internal(format!("end of channel: {}", e)));
                                },
                            }
                        },
                    };

                    let block = match block_result {
                        Ok(b) => b,
                        Err(_) => {
                            block_failed = true;
                            break;
                        }
                    };

                    let block_real_size = block.real_size();
                    let compressed = block.compressed();

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
                        Err(e) => return Err(SDError::CryptoError(Box::new(e))),
                    };
                    let block_padded_size = wrapped_block.len() as u64;

                    let padding_overhead = if compressed {
                        block_padded_size - block_compressed_size
                    } else {
                        block_padded_size - block_real_size
                    };

                    item_padding += padding_overhead;

                    let cache_message = ::cache::WriteCacheMessage::new(Some(wrapped_block), false, None);

                    match block_send.send(cache_message) {
                        Ok(()) => {

                        },
                        Err(e) => {
                            issue(&format!("not able to sync file {}: writing failed ({})", full_path.display(), e));
                            block_failed = true;
                            break;
                        },
                    }
                }

                if block_failed {
                    issue(&format!("not able to sync file {}: could not read from file", full_path.display()));

                    failed = failed +1;
                    continue;
                }

                processed_size_padding += item_padding;


                let stats = block_generator.stats();
                if DEBUG_STATISTICS {
                    let compression_ratio = (stats.processed_size_compressed as f64 / stats.processed_size as f64 ) * 100.0;

                    debug!("{} chunks", stats.discovered_chunk_count);
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
            match ::std::fs::read_link(&full_path) {
                Ok(path) => {
                    match  header.set_link_name(path) {
                        Ok(()) => {

                        },
                        Err(e) => {
                            issue(&format!("failed to set symlink for {}: {}", full_path.display(), e));
                        }
                    };
                },
                Err(e) => {
                    issue(&format!("failed to set symlink for {}: {}", full_path.display(), e));
                }
            };

            header.set_size(0); /// hmac list size is zero when file has no actual data
            header.set_cksum();
            ar.append(&header, hmac_bag.as_slice()).expect("failed to append symlink to archive header");
        }
    }

    debug!("signaling write cache we're finished");

    let cache_message = ::cache::WriteCacheMessage::new(None, false, None);

    match block_send.send(cache_message) {
        Ok(()) => {

        },
        Err(e) => {
            issue(&format!("not able to signal write cache to finish: {}", e));
        },
    }

    debug!("waiting for write cache to finish");

    match write_thread.join() {
        Ok(()) => {},
        Err(_) => {
            debug!("thread join failed");
        }
    }
    debug!("processing session and statistics");

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
        Err(e) => return Err(SDError::CryptoError(Box::new(e))),
    };

    let mut s: Vec<WrappedSyncSession> = Vec::new();

    s.push(wrapped_session);

    debug!("finishing sync session");

    let mut should_retry = true;
    let mut retries_left = 15.0;

    while should_retry {
        /// allow caller to tick the progress display, if one exists
        progress(estimated_size, processed_size, 0, percent_completed, false);

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
            debug!("backing off for {:?}", delay);

            thread::sleep(delay);
        }

        match finish_sync_session(&token, folder_id, true, &s, processed_size as usize) {
            Ok(()) => {
                should_retry = false;
            },
            Err(SDAPIError::Authentication) => {
                return Err(SDError::Authentication)
            },
            Err(e) => {
                retries_left = retries_left - 1.0;
                if retries_left <= 0.0 {
                    issue(&format!("not able to finish sync: too many retries ({})", e));
                    return Err(SDError::ExceededRetries(15));
                }
            }
        };
    }


    progress(estimated_size, processed_size, 0, 100.0, false);

    Ok(())
}


pub fn restore(token: &Token,
               session_name: &str,
               main_key: &Key,
               folder_id: u64,
               destination: PathBuf,
               session_size: u64,
               progress: &mut FnMut(u64, u64, u64, f64, bool),
               issue: &mut FnMut(&str)) -> Result<(), SDError> {
    let restore_start_time = ::std::time::Instant::now();

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

    let read_session_start_time = ::std::time::Instant::now();

    let session_body = match read_session(token, folder_id, session_name, true) {
        Ok(session_data) => session_data,
        Err(e) => return Err(SDError::from(e))
    };
    trace!("Session reading took {} seconds", read_session_start_time.elapsed().as_secs());


    debug!("restoring session for: {} (folder id {})", folder_name, folder_id);


    let session_processing_start_time = ::std::time::Instant::now();

    let w_session = match WrappedSyncSession::from(session_body) {
        Ok(ws) => ws,
        Err(e) => return Err(e),
    };
    trace!("Session processing took {} seconds", session_processing_start_time.elapsed().as_secs());


    let session_unwrap_start_time = ::std::time::Instant::now();

    let session = match w_session.to_session(main_key) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    trace!("Session unwrapping took {} seconds", session_unwrap_start_time.elapsed().as_secs());

    let mut processed_size: u64 = 0;

    let mut ar = Archive::new(session.as_ref());

    let mut failed = 0;

    let archive_reading_start_time = ::std::time::Instant::now();

    for item in ar.entries().unwrap() {
        if is_sync_task_cancelled(session_name.to_owned()) {
            return Err(SDError::Cancelled)
        }
        let restore_item_start_time = ::std::time::Instant::now();

        let percent_completed: f64 = (processed_size as f64 / session_size as f64) * 100.0;


        let mut file_entry = match item {
            Ok(e) => e,
            Err(e) => {
                issue(&format!("not able to restore session entry: {}", e));
                failed + failed + 1;
                continue // we do care about errors here, but we can't really recover from them for this item
            }
        };

        let mut full_path = PathBuf::from(&destination);

        match file_entry.path() {
            Ok(ref entry_path) => {
                debug!("examining {}", &entry_path.display());

                full_path.push(entry_path);
            }
            Err(e) => {
                issue(&format!("cannot restore invalid path in session {}:", e));
                progress(session_size, processed_size, 0, percent_completed, false);

                failed + failed + 1;
                continue // we do care about errors here, but we can't really recover from them for this item
            }
        };


        /// call out to the library user with progress



        let entry_type = file_entry.header().entry_type();

        /// process if not a directory or socket
        match entry_type {
            EntryType::Regular => {
                let f = match File::create(&full_path) {
                    Ok(file) => file,
                    Err(err) => {
                        issue(&format!("not able to create file at {}: {}", full_path.display(), err));

                        failed = failed +1;
                        continue
                    },
                };

                let stream_length = file_entry.header().size().unwrap();

                trace!("entry has {} blocks", stream_length / 32);
                
                if stream_length > 0 {
                    let mut stream = BufWriter::new(f);

                    let mut block_hmac_bag = Vec::new();

                    try!(file_entry.read_to_end(&mut block_hmac_bag));
                    let block_hmac_list = match ::binformat::parse_hmacs(&block_hmac_bag) {
                        Done(_, o) => o,
                        Error(e) => {
                            error!("hmac bag parsing failed: {}", e);

                            return Err(SDError::SessionUnreadable)
                        },
                        Incomplete(_) => panic!("should never happen")
                    };


                    for block_hmac in block_hmac_list.iter() {
                        if is_sync_task_cancelled(session_name.to_owned()) {
                            return Err(SDError::Cancelled)
                        }

                        let block_start_time = ::std::time::Instant::now();

                        /// allow caller to tick the progress display, if one exists
                        progress(session_size, processed_size, 0, percent_completed, true);

                        let mut should_retry = true;
                        let mut retries_left = 15.0;

                        let block_hmac_hex = block_hmac.to_hex();
                        debug!("processing block {}", &block_hmac_hex);


                        let mut wrapped_block: Option<WrappedBlock> = None;
                        let block_read_start_time = ::std::time::Instant::now();

                        /// get block from cache if possible
                        match ::cache::read_block(&block_hmac_hex) {
                            Ok(br) => {
                                should_retry = false;
                                wrapped_block = Some(br);
                                debug!("cache provided block: {}", &block_hmac_hex);
                            },
                            _ => {}
                        };

                        while should_retry {
                            if is_sync_task_cancelled(session_name.to_owned()) {
                                return Err(SDError::Cancelled)
                            }

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
                                debug!("backing off for {:?}", delay);

                                thread::sleep(delay);
                            }

                            /// get block from the server

                            match ::sdapi::read_block(&token, &block_hmac_hex) {
                                Ok(rb) => {
                                    trace!("Block read took {} seconds", block_read_start_time.elapsed().as_secs());

                                    /// allow caller to tick the progress display, if one exists
                                    progress(session_size, processed_size, 0, percent_completed, false);

                                    should_retry = false;
                                    debug!("server provided block: {}", &block_hmac_hex);
                                    let block_processing_start_time = ::std::time::Instant::now();

                                    let wb = match WrappedBlock::from(rb, (&block_hmac_hex).from_hex().unwrap()) {
                                        Ok(wb) => wb,
                                        Err(e) => {
                                            debug!("block failed validation: {}", &block_hmac_hex);

                                            return Err(e)
                                        },
                                    };
                                    trace!("Block processing took {} seconds", block_processing_start_time.elapsed().as_secs());

                                    debug!("block processed: {}", &block_hmac_hex);

                                    let block_cache_write_time = ::std::time::Instant::now();

                                    match ::cache::write_binary(&wb) {
                                        _ => {}
                                    };
                                    trace!("Block write to cache took {} seconds", block_cache_write_time.elapsed().as_secs());

                                    wrapped_block = Some(wb);
                                },
                                Err(SDAPIError::Authentication) => {
                                    return Err(SDError::Authentication)
                                },
                                Err(SDAPIError::RequestFailed(err)) => {
                                    retries_left = retries_left - 1.0;

                                    progress(session_size, processed_size, 0, percent_completed, false);


                                    if retries_left <= 0.0 {
                                        issue(&format!("not able to retrieve part of {}: {}", full_path.display(), err.description()));

                                        return Err(SDError::RequestFailure(err))
                                    }
                                },
                                _ => {}
                            };


                        }

                        let wrapped_block_s = wrapped_block.unwrap();
                        let block_unwrap_time = ::std::time::Instant::now();

                        let block = match wrapped_block_s.to_block(main_key) {
                            Ok(b) => b,
                            Err(e) => return Err(e),
                        };
                        trace!("Block unwrapping took {} seconds", block_unwrap_time.elapsed().as_secs());

                        debug!("block unwrapped");

                        let block_write_time = ::std::time::Instant::now();

                        {
                            let new_position = stream.seek(SeekFrom::Current(0)).unwrap();
                            trace!("write start {:?}", new_position);

                        }
                        trace!("writing block segment of {} bytes", block.len());

                        {
                            try!(stream.write_all(block.as_ref()));
                        }
                        {
                            let new_position = stream.seek(SeekFrom::Current(0)).unwrap();
                            trace!("new position {:?}", new_position);

                        }
                        trace!("Block write took {} seconds", block_write_time.elapsed().as_secs());

                        trace!("Block total handling time took {} seconds", block_start_time.elapsed().as_secs());

                        processed_size += block.len() as u64;
                        progress(session_size, processed_size, block.len() as u64, percent_completed, false);

                    }

                } else {
                    // empty file, just write one out with the same metadata but no body

                }
            },
            EntryType::Directory => {
                try!(fs::create_dir_all(&full_path));
            },
            EntryType::Link => {
                let src = match file_entry.link_name() {
                    Ok(op) => {
                        match op {
                            Some(pa) => pa,
                            None => {
                                issue(&format!("not able to restore hard link {}: no link destination found", full_path.display()));
                                continue;
                            },
                        }
                    },
                    Err(e) => {
                        issue(&format!("not able to restore hard link {}: {}", full_path.display(), e));
                        continue;
                    }
                };


                match ::std::fs::hard_link(&src, &full_path) {
                    Ok(()) => {},
                    Err(e) => {
                        issue(&format!("not able to restore hard link {}: {}", full_path.display(), e));
                    },
                }

            },
            EntryType::Symlink => {
                let src = match file_entry.link_name() {
                    Ok(op) => {
                        match op {
                            Some(pa) => pa,
                            None => {
                                issue(&format!("not able to restore symlink {}: no link destination found", full_path.display()));
                                continue;
                            },
                        }
                    },
                    Err(e) => {
                        issue(&format!("not able to restore symlink {}: {}", full_path.display(), e));
                        continue;
                    }
                };

                if src.iter().count() == 0 {
                    issue(&format!("not able to restore symlink {}: no link destination found", full_path.display()));
                    continue;
                }

                #[cfg(unix)]
                match ::std::os::unix::fs::symlink(&src, &full_path) {
                    Ok(()) => {},
                    Err(e) => {
                        issue(&format!("not able to restore symlink {}: {}", full_path.display(), e));

                    },
                }

                #[cfg(windows)] {
                    let md = match ::std::fs::metadata(&src) {
                        Ok(m) => m,
                        Err(e) => {
                            issue(&format!("not able to restore symlink {}: {}", full_path.display(), e));
                            continue
                        },
                    };

                    let is_dir = md.file_type().is_dir();
                    
                    if is_dir {
                        match ::std::os::windows::fs::symlink_dir(&src, full_path) {
                            Ok(()) => {},
                            Err(e) => {
                                issue(&format!("not able to restore directory symlink {}: {}", full_path.display(), e));
                            },
                        }
                    } else {
                        match ::std::os::windows::fs::symlink_file(&src, full_path) {
                            Ok(()) => {},
                            Err(e) => {
                                issue(&format!("not able to restore file symlink {}: {}", full_path.display(), e));
                            },
                        }
                    }
                }


            },
            _ => {
                issue(&format!("not able to restore {:?} file {}: unsupported file type", entry_type, full_path.display()));
            },
        }

        trace!("Restore item total handling time took {} seconds", restore_item_start_time.elapsed().as_secs());
    }
    trace!("Archive read total time took {} seconds", archive_reading_start_time.elapsed().as_secs());

    trace!("Restore total time took {} seconds", restore_start_time.elapsed().as_secs());

    debug!("restoring session finished");

    progress(session_size, processed_size, 0, 100.0, false);

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