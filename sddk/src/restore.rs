use std::str;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read, Write, Seek, SeekFrom};
use std::{thread, time};

/// external crate imports
use rustc_serialize::hex::{ToHex, FromHex};
use rand::distributions::{IndependentSample, Range};
use tar::{Archive, EntryType};
use nom::IResult::*;

/// internal imports

use models::*;
use block::*;
use sdapi::*;
use keys::*;

use core::get_sync_folder;


#[cfg(feature = "locking")]
use lock::FolderLock;

use error::{SDAPIError, SDError};

use session::{WrappedSyncSession};

use sync_state::is_sync_task_cancelled;


pub fn restore(token: &Token,
               session_name: &str,
               main_key: &Key,
               folder_id: u64,
               destination: PathBuf,
               session_size: u64,
               progress: &mut FnMut(u64, u64, u64, f64, bool),
               issue: &mut FnMut(&str)) -> Result<(), SDError> {
    let restore_start_time = ::std::time::Instant::now();

    fs::create_dir_all(&destination)?;

    let dpath: &Path = &destination;
    let path_exists = dpath.exists();
    let path_is_dir = dpath.is_dir();

    if !path_exists || !path_is_dir {
        return Err(SDError::FolderMissing);
    }

    let folder = match get_sync_folder(token, folder_id) {
        Ok(folder) => folder,
        Err(e) => return Err(SDError::from(e)),
    };

    let folder_name = &folder.folderName;

    #[cfg(feature = "locking")]
    let flock = FolderLock::new(&destination)?;

    #[cfg(feature = "locking")]
    defer!({
        /// try to ensure the folder goes back to unlocked state, but if not there isn't anything
        /// we can do about it
        flock.unlock();
    });

    let read_session_start_time = ::std::time::Instant::now();

    let session_body = match read_session(token, folder_id, session_name, true) {
        Ok(session_data) => session_data,
        Err(e) => return Err(SDError::from(e)),
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
            issue(&format!("sync cancelled ({})", session_name));

            return Err(SDError::Cancelled);
        }
        let restore_item_start_time = ::std::time::Instant::now();

        let percent_completed: f64 = (processed_size as f64 / session_size as f64) * 100.0;


        let mut file_entry = match item {
            Ok(e) => e,
            Err(e) => {
                issue(&format!("not able to restore session entry: {}", e));
                failed + failed + 1;
                continue; // we do care about errors here, but we can't really recover from them for this item
            },
        };

        let mut full_path = PathBuf::from(&destination);

        match file_entry.path() {
            Ok(ref entry_path) => {
                debug!("examining {}", &entry_path.display());

                full_path.push(entry_path);
            },
            Err(e) => {
                issue(&format!("cannot restore invalid path in session {}:", e));
                progress(session_size, processed_size, 0, percent_completed, false);

                failed + failed + 1;
                continue; // we do care about errors here, but we can't really recover from them for this item
            },
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
                        continue;
                    },
                };

                let stream_length = file_entry.header().size().unwrap();

                trace!("entry has {} blocks", stream_length / 32);

                if stream_length > 0 {
                    let mut stream = BufWriter::new(f);

                    let mut block_hmac_bag = Vec::new();

                    file_entry.read_to_end(&mut block_hmac_bag)?;
                    let block_hmac_list = match ::binformat::parse_hmacs(&block_hmac_bag) {
                        Done(_, o) => o,
                        Error(e) => {
                            error!("hmac bag parsing failed: {}", e);

                            return Err(SDError::SessionUnreadable);
                        },
                        Incomplete(_) => panic!("should never happen"),
                    };


                    for block_hmac in block_hmac_list.iter() {
                        if is_sync_task_cancelled(session_name.to_owned()) {
                            issue(&format!("sync cancelled ({})", session_name));

                            return Err(SDError::Cancelled);
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
                            _ => {},
                        };

                        while should_retry {
                            if is_sync_task_cancelled(session_name.to_owned()) {
                                issue(&format!("sync cancelled ({})", session_name));

                                return Err(SDError::Cancelled);
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

                            match ::sdapi::read_block(token, &block_hmac_hex) {
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

                                            return Err(e);
                                        },
                                    };
                                    trace!("Block processing took {} seconds", block_processing_start_time.elapsed().as_secs());

                                    debug!("block processed: {}", &block_hmac_hex);

                                    let block_cache_write_time = ::std::time::Instant::now();

                                    match ::cache::write_binary(&wb) {
                                        _ => {},
                                    };
                                    trace!("Block write to cache took {} seconds", block_cache_write_time.elapsed().as_secs());

                                    wrapped_block = Some(wb);
                                },
                                Err(SDAPIError::Authentication) => return Err(SDError::Authentication),
                                Err(SDAPIError::RequestFailed(err)) => {
                                    retries_left = retries_left - 1.0;

                                    progress(session_size, processed_size, 0, percent_completed, false);


                                    if retries_left <= 0.0 {
                                        issue(&format!("not able to retrieve part of {}: {}", full_path.display(), err.description()));

                                        return Err(SDError::RequestFailure(err));
                                    }
                                },
                                _ => {},
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
                            stream.write_all(block.as_ref())?;
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
                fs::create_dir_all(&full_path)?;
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
                    },
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
                    },
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
                            continue;
                        },
                    };

                    let is_dir = md.file_type().is_dir();

                    if is_dir {
                        match ::std::os::windows::fs::symlink_dir(&src, &full_path) {
                            Ok(()) => {},
                            Err(e) => {
                                issue(&format!("not able to restore directory symlink {}: {}", full_path.display(), e));
                            },
                        }
                    } else {
                        match ::std::os::windows::fs::symlink_file(&src, &full_path) {
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