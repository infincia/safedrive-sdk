use std::str;
use std::path::{Path, PathBuf};
use std::{thread, time};

/// external crate imports
use tar::{Builder, Header};
use walkdir::WalkDir;

/// internal imports

use models::*;
use constants::*;
use sdapi::*;
use keys::*;
use binformat::BinaryWriter;

use core::get_sync_folder;

#[cfg(feature = "locking")]
use lock::FolderLock;

use error::{SDAPIError, SDError};
use SYNC_VERSION;

use session::{SyncSession, WrappedSyncSession};

use sync_state::is_sync_task_cancelled;

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
        Err(e) => return Err(SDError::from(e)),
    };

    let folder_path = PathBuf::from(&folder.folderPath);
    let folder_name = &folder.folderName;

    let p: &Path = &folder_path;
    let path_exists = p.exists();
    let path_is_dir = p.is_dir();

    if !path_exists || !path_is_dir {
        return Err(SDError::FolderMissing);
    }

    #[cfg(feature = "locking")]
    let flock = FolderLock::new(&folder_path)?;

    #[cfg(feature = "locking")]
    defer!({
        /// try to ensure the folder goes back to unlocked state, but if not there isn't anything
        /// we can do about it
        flock.unlock();
    });

    match register_sync_session(token, folder_id, session_name, true) {
        Ok(()) => {},
        Err(e) => return Err(SDError::from(e)),
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
                continue;
            },
        };

        let stream_length = md.len();
        trace!("estimating size of {}... OK, {}", item_path.display(), stream_length);

        estimated_size = estimated_size + stream_length;
    }

    let item_limit = 300;
    let size_limit = 10_000_000;

    let write_cache: ::cache::WriteCache = ::cache::WriteCache::new(item_limit, size_limit);

    let (block_send, status_receive) = write_cache.upload_thread(token, session_name);

    let mut failed = 0;

    let mut percent_completed: f64 = (processed_size as f64 / estimated_size as f64) * 100.0;

    for item in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
        if is_sync_task_cancelled(session_name.to_owned()) {
            issue(&format!("sync cancelled ({})", session_name));

            let cache_message = ::cache::WriteCacheMessage::new(None, true, None);

            match block_send.send(cache_message) {
                Ok(()) => {

                },
                Err(e) => {
                    issue(&format!("not able to cancel upload: ({})", e));
                },
            }
            return Err(SDError::Cancelled);
        }

        trace!("examining {}", item.path().display());

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
            Err(_) => {
                failed = failed + 1;
                continue;
            },
        };

        let stream_length = md.len();
        let is_file = md.file_type().is_file();
        let is_dir = md.file_type().is_dir();
        let is_symlink = md.file_type().is_symlink();

        /// store metadata for directory or file
        let mut header = Header::new_gnu();
        header.set_metadata(&md);

        let mut hmac_bag: Vec<u8> = Vec::new();

        // chunk file if not a directory or socket
        if is_file {
            if stream_length > 0 {

                let mut block_generator = ::chunk::BlockGenerator::new(&full_path,
                                                                       main_key,
                                                                       hmac_key,
                                                                       tweak_key,
                                                                       stream_length,
                                                                       SYNC_VERSION);

                let mut item_padding: u64 = 0;

                let mut block_failed = false;

                for block_result in block_generator.by_ref() {
                    if is_sync_task_cancelled(session_name.to_owned()) {
                        issue(&format!("sync cancelled ({})", session_name));

                        let cache_message = ::cache::WriteCacheMessage::new(None, true, None);

                        match block_send.send(cache_message) {
                            Ok(()) => {

                            },
                            Err(e) => {
                                issue(&format!("not able to cancel upload: ({})", e));
                            },
                        }
                        return Err(SDError::Cancelled);
                    }

                    match status_receive.try_recv() {
                        Ok(msg) => {
                            match msg {
                                Ok(_) => {},
                                Err(e) => return Err(e),
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
                        },
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

                    processed_size += block_real_size as u64;

                    progress(estimated_size, processed_size, block_real_size as u64, percent_completed, false);

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

                    trace!("{} chunks", stats.discovered_chunk_count);
                    trace!("average size: {} bytes", stats.processed_size / stats.discovered_chunk_count);
                    trace!("compression: {}/{} ({}%)", stats.processed_size_compressed, stats.processed_size, compression_ratio);

                    let padding_ratio = (item_padding as f64 / stats.processed_size_compressed as f64 ) * 100.0;

                    trace!("padding overhead: {} ({}%)", item_padding, padding_ratio);

                    trace!("hmac bag has: {} ids <{} bytes>", hmac_bag.len() / 32, stats.discovered_chunk_count * 32);
                    trace!("expected chunk size: {} bytes", stats.discovered_chunk_expected_size);
                    trace!("smallest chunk: {} bytes", stats.discovered_chunk_smallest_size);
                    trace!("largest chunk: {} bytes", stats.discovered_chunk_largest_size);
                    trace!("standard size deviation: {} bytes", (stats.discovered_chunk_size_variance as f64 / stats.discovered_chunk_count as f64).sqrt() as u64);
                }

                assert!(stats.processed_size == stream_length);
                trace!("calculated {} real bytes of blocks, matching stream size {}", stats.processed_size, stream_length);

                header.set_size(stats.discovered_chunk_count * HMAC_SIZE as u64); /// hmac list size
                header.set_cksum();

                ar.append_data(&mut header, &relative_path, hmac_bag.as_slice()).expect("failed to append session entry header");

            } else {
                header.set_size(0); /// hmac list size is zero when file has no actual data
                header.set_cksum();

                ar.append_data(&mut header, &relative_path, hmac_bag.as_slice()).expect("failed to append zero length archive header");
            }
        } else if is_dir {
            /// folder
            header.set_size(0); /// hmac list size is zero when file has no actual data
            header.set_cksum();

            ar.append_data(&mut header, &relative_path, hmac_bag.as_slice()).expect("failed to append folder to archive header");
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
                        },
                    };
                },
                Err(e) => {
                    issue(&format!("failed to set symlink for {}: {}", full_path.display(), e));
                },
            };

            header.set_size(0); /// hmac list size is zero when file has no actual data
            header.set_cksum();

            ar.append_data(&mut header, &relative_path, hmac_bag.as_slice()).expect("failed to append symlink to archive header");
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

    loop {
        match status_receive.try_recv() {
            Ok(msg) => {
                match msg {
                    Ok(finished) => {
                        if finished {
                            debug!("write thread says it's finished, continuing");
                            break;
                        }
                    },
                    Err(e) => return Err(e),
                };
            },
            Err(e) => {
                match e {
                    ::std::sync::mpsc::TryRecvError::Empty => {},
                    ::std::sync::mpsc::TryRecvError::Disconnected => {
                        debug!("write thread has disconnected, continuing");

                        break;
                    },
                }
            },
        };
        let delay = time::Duration::from_millis(500);

        thread::sleep(delay);
    }

    debug!("processing session and statistics");

    /// since we're writing to a buffer in memory there shouldn't be any errors here...
    /// unless the system is also completely out of memory, but there's nothing we can do about that,
    /// so if it proves to be an issue we'll have to look for anything else that might use a lot of
    /// memory and use temp files instead, where possible
    let raw_session = ar.into_inner().unwrap();


    let session = SyncSession::new(SYNC_VERSION,
                                   folder_id,
                                   session_name.to_string(),
                                   Some(processed_size),
                                   None,
                                   raw_session);


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
            estimated_size += size;

            debug!("session file total compressed: {}", size);
            debug!("session file compression ratio: {}", compression_ratio);
        },
        None => {},
    }


    let wrapped_session = match session.to_wrapped(main_key) {
        Ok(ws) => ws,
        Err(e) => return Err(SDError::CryptoError(Box::new(e))),
    };

    let mut s: Vec<WrappedSyncSession> = Vec::new();

    s.push(wrapped_session);

    debug!("finishing sync session");

    /// allow caller to tick the progress display, if one exists
    progress(estimated_size, processed_size, 0, percent_completed, false);

    match finish_sync_session(token, folder_id, true, &s, processed_size as usize, move |total, current, new| {
        debug!("session upload progress: {}/{}, {} new", current, total, new);
    }) {
        Ok(()) => {},
        Err(SDAPIError::Authentication) => return Err(SDError::Authentication),
        Err(e) => {
            issue(&format!("not able to finish sync: {}", e));
            return Err(SDError::RequestFailure(Box::new(e)));
        },
    };

    progress(estimated_size, processed_size, 0, 100.0, false);

    Ok(())
}