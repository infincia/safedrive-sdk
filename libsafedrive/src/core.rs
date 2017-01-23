use std;
use std::str;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use std::cmp::{min, max};
use std::{thread, time};

// external crate imports

use ::rustc_serialize::hex::{ToHex};
use ::rand::distributions::{IndependentSample, Range};
use ::tar::{Builder, Header, Archive, EntryType};
use ::walkdir::WalkDir;
use ::cdc::*;
use ::nom::IResult::*;

// internal imports

use ::models::*;
use ::block::*;
use ::constants::*;
use ::sdapi::*;
use ::keys::*;
use ::error::{CryptoError, SDAPIError, SDError};
use ::CONFIGURATION;
use ::CACHE_DIR;
use ::session::{SyncSession, WrappedSyncSession};

// crypto exports

pub fn sha256(input: &[u8]) -> String {
    ::util::sha256(input)
}

// helpers

pub fn get_app_directory(config: &Configuration) -> Result<PathBuf, String> {
    ::util::get_app_directory(config)
}

pub fn unique_client_hash(email: &str) -> Result<String, String> {
    ::util::unique_client_hash(email)
}

// internal functions

pub fn initialize<'a>(local_storage_path: &'a Path, config: Configuration) {
    let mut c = CONFIGURATION.write().unwrap();
    *c = config;

    if !::sodiumoxide::init() == true {
        panic!("sodium initialization failed, cannot continue");
    }

    if let Err(e) = fs::create_dir_all(&local_storage_path) {
        panic!("failed to create local directories: {}", e);
    }
    let mut p = PathBuf::from(local_storage_path);
    p.push("cache");

    let cache_s = match p.as_path().to_str() {
        Some(ref s) => s.to_string(),
        None => {
            panic!("failed to create local cache dir");
        }
    };
    if let Err(e) = fs::create_dir_all(&cache_s) {
        panic!("failed to create local cache dir: {}", e);
    }
    let mut cd = CACHE_DIR.write().unwrap();
    *cd = cache_s;


    let sodium_version = ::sodiumoxide::version::version_string();
    debug!("libsodium {}", sodium_version);

    #[cfg(target_os = "linux")]
    let ssl_version = ::openssl::version::version();
    #[cfg(target_os = "linux")]
    debug!("{}>", ssl_version);

    debug!("ready");
}

pub fn login(unique_client_id: &str,
             username: &str,
             password:  &str) -> Result<(Token, AccountStatus, UniqueClientID), SDError> {

    match register_client(unique_client_id, username, password) {
        Ok((t, ucid)) => {
            match account_status(&t) {
                Ok(s) => return Ok((t, s, ucid)),
                Err(e) => Err(SDError::from(e))
            }
        },
        Err(e) => return Err(SDError::from(e))
    }
}

pub fn load_keys(token: &Token, recovery_phrase: Option<String>, store_recovery_key: &Fn(&str)) -> Result<Keyset, SDError> {
    // generate new keys in all cases, the account *may* already have some stored, we only
    // find out for sure while trying to store them.
    //
    // we do this on purpose:
    //
    //    1. to eliminate possible race conditions where two clients generate valid keysets and try
    //       to store them at the same time, only the server knows who won and we only need one HTTP
    //       request to find out
    //
    //    2. clients are never allowed to supply their own recovery phrases and keys to the SDK.
    //       either we actually need new keys, in which case the ones we safely generated internally
    //       are stored on the server immediately, or we're wrong and the server tells us what
    //       to use instead
    //
    //
    // in all cases, the library will only use keys that have been returned by the server, this
    // guarantees that the keys we actually use are correct as long as the server is correct
    //
    let new_wrapped_keyset = match WrappedKeyset::new() {
        Ok(wks) => wks,
        Err(_) => return Err(SDError::from(CryptoError::KeyGenerationFailed))
    };


    match account_key(token, &new_wrapped_keyset) {
        Ok(real_wrapped_keyset) => {
            // now we check to see if the keys returned by the server match the existing phrase or not

            // if we were given an existing phrase try it, otherwise try the new one
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
                        // a new keyset was generated so we must return the phrase to the caller so it
                        // can be stored and displayed
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
                       folder_id: u32) -> Result<RegisteredFolder, SDError> {
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
                       path: &str) -> Result<u32, SDError> {
    match create_folder(token, path, name, true) {
        Ok(folder_id) => Ok(folder_id),
        Err(e) => Err(SDError::from(e))
    }
}

pub fn remove_sync_folder(token: &Token,
                          folder_id: u32) -> Result<(), SDError> {
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
                            folder_id: u32,
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

pub fn sync(token: &Token,
            session_name: &str,
            main_key: &Key,
            hmac_key: &Key,
            tweak_key: &Key,
            folder_id: u32,
            progress: &mut FnMut(u32, u32, u32, f64, bool)) -> Result<(), SDError> {

    let folder = match get_sync_folder(token, folder_id) {
        Ok(folder) => folder,
        Err(e) => return Err(SDError::from(e))
    };

    match register_sync_session(token, folder_id, session_name, true) {
        Ok(()) => {},
        Err(e) => return Err(SDError::from(e))
    };

    let folder_path = PathBuf::from(&folder.folderPath);
    let folder_name = &folder.folderName;


    let archive_file = Vec::new();

    if DEBUG_STATISTICS {
        debug!("creating archive for: {} (folder id {})", folder_name, folder_id);
    }

    let mut ar = Builder::new(archive_file);
    let mut archive_size: u64 = 0;

    let mut estimated_size: u64 = 0;

    for item in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
        if DEBUG_STATISTICS {
            debug!("estimating size of {}...", item.path().display());
        }
        let p = item.path();

        let f = match File::open(p) {
            Ok(file) => file,
            Err(_) => { continue },
        };
        let md = match f.metadata() {
            Ok(m) => m,
            Err(_) => { continue },
        };

        let stream_length = md.len();
        if DEBUG_STATISTICS {
            debug!("stream: {}", stream_length);
        }
        estimated_size = estimated_size + stream_length;
    }

    let mut failed = 0;

    let mut completed_count = 0.0;
    for item in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
        if DEBUG_STATISTICS {
            debug!("examining {}", item.path().display());
        }
        let percent_completed: f64 = (archive_size as f64 / estimated_size as f64) * 100.0;

        // call out to the library user with progress
        progress(estimated_size as u32, archive_size as u32, 0 as u32, percent_completed, false);

        completed_count = completed_count + 1.0;


        let p = item.path();
        let p_relative = p.strip_prefix(&folder_path).expect("failed to unwrap relative path");

        let f = match File::open(p) {
            Ok(file) => file,
            Err(_) => { failed = failed +1; continue },
        };
        let md = match f.metadata() {
            Ok(m) => m,
            Err(_) => { failed = failed +1; continue },
        };

        let stream_length = md.len();
        let is_file = md.file_type().is_file();
        let is_dir = md.file_type().is_dir();

        // store metadata for directory or file
        let mut header = Header::new_ustar();
        if let Err(err) = header.set_path(p_relative) {
            if DEBUG_STATISTICS {
                debug!("not adding invalid path: '{}' (reason: {})", p_relative.display(), err);
            }
            failed + failed + 1;
            continue // we don't care about errors here, they'll only happen for truly invalid paths
        }
        header.set_metadata(&md);

        // chunk file if not a directory or socket
        if is_file {
            if stream_length > 0 {
                let mut f_chunk = File::open(p).expect("failed to open file to retrieve chunk");

                let reader: BufReader<File> = BufReader::new(f);
                let byte_iter = reader.bytes().map(|b| b.expect("failed to unwrap block data"));

                let separator_size_nb_bits: u32 = 6;

                let t = tweak_key.as_ref();

                #[inline]
                fn chunk_predicate(x: u64) -> bool {
                    const BITMASK: u64 = (1u64 << 18) - 1;
                    x & BITMASK == BITMASK
                }

                let separator_iter = SeparatorIter::custom_new(byte_iter, separator_size_nb_bits, chunk_predicate);
                let chunk_iter = ChunkIter::new(separator_iter, stream_length);
                let mut nb_chunk = 0;
                let mut total_size = 0;
                let mut skipped_blocks = 0;
                let mut smallest_size = std::u64::MAX;
                let mut largest_size = 0;
                let expected_size = 1 << 18;
                let mut size_variance = 0;
                let mut chunks: Vec<u8> = Vec::new();

                let mut chunk_start = 0;

                for chunk in chunk_iter {
                    // allow caller to tick the progress display, if one exists
                    progress(estimated_size as u32, archive_size as u32, 0 as u32, percent_completed, false);

                    nb_chunk += 1;
                    total_size += chunk.size;
                    archive_size += chunk.size;

                    debug!("creating chunk at {} of size {}", chunk_start, chunk.size);

                    smallest_size = min(smallest_size, chunk.size);
                    largest_size = max(largest_size, chunk.size);
                    size_variance += (chunk.size as i64 - expected_size as i64).pow(2);
                    f_chunk.seek(SeekFrom::Start(chunk_start)).expect("failed to seek chunk reader");

                    let mut buffer = BufReader::new(&f_chunk).take(chunk.size);
                    let mut data: Vec<u8> = Vec::with_capacity(100000); //expected size of largest block

                    chunk_start = chunk_start + chunk.size;

                    if let Err(e) = buffer.read_to_end(&mut data) {
                        return Err(SDError::from(e))
                    }

                    let raw_chunk = data.as_slice();


                    let main_key = ::sodiumoxide::crypto::secretbox::Key::from_slice(main_key.as_ref())
                        .expect("failed to get main key struct");

                    // calculate hmac of the block
                    let hmac_key = ::sodiumoxide::crypto::auth::Key::from_slice(hmac_key.as_ref())
                        .expect("failed to get hmac key struct");

                    let block_hmac = ::sodiumoxide::crypto::auth::authenticate(raw_chunk, &hmac_key);
                    chunks.extend_from_slice(block_hmac.as_ref());

                    let block_name = block_hmac.as_ref().to_hex();

                    let mut should_retry = true;
                    let mut retries_left = 15.0;
                    let mut potentially_uploaded_data: Option<Vec<u8>> = None;

                    // generate a new chunk key once in case we need it later. this is cheap to do
                    let block_key_raw = ::sodiumoxide::randombytes::randombytes(KEY_SIZE);

                    while should_retry {
                        // allow caller to tick the progress display, if one exists
                        progress(estimated_size as u32, archive_size as u32, 0 as u32, percent_completed, false);

                        let failed_count = 15.0 - retries_left;
                        let mut rng = ::rand::thread_rng();

                        // we pick a multiplier randomly to avoid a bunch of clients trying again
                        // at the same 2/4/8/16 backoff times time over and over if the server
                        // is overloaded or down
                        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

                        if failed_count >= 2.0 && retries_left > 0.0 {
                            // back off significantly every time a call fails but only after the
                            // second try, the first failure could be us not including the data
                            // when we should have
                            let backoff_time = backoff_multiplier * (failed_count * failed_count);
                            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
                            thread::sleep(delay);
                        }

                        // tell the server to mark this block without the data first, if that fails we try uploading

                        match write_block(&token, session_name, &block_name, &potentially_uploaded_data) {
                            Ok(()) => {
                                skipped_blocks = skipped_blocks + 1;
                                // allow caller to tick the progress display, if one exists
                                progress(estimated_size as u32, archive_size as u32, chunk.size as u32, percent_completed, false);
                                should_retry = false
                            },
                            Err(SDAPIError::RequestFailed(err)) => {
                                retries_left = retries_left - 1.0;
                                if retries_left <= 0.0 {
                                    // TODO: pass better error info up the call chain here rather than a failure
                                    return Err(SDError::RequestFailure(err))
                                }
                            },
                            Err(SDAPIError::RetryUpload) => {
                                retries_left = retries_left - 1.0;

                                match potentially_uploaded_data {
                                    Some(_) => {},
                                    None => {

                                        let block_key_struct = ::sodiumoxide::crypto::secretbox::Key::from_slice(&block_key_raw)
                                        .expect("failed to get block key struct");

                                        // We use the first 24 bytes of the block hmac value as nonce for wrapping
                                        // the block key and encrypting the block itself.
                                        //
                                        // This is cryptographically safe but still deterministic: encrypting
                                        // the same block twice with a specific key will always produce the same
                                        // output block, which is critical for versioning and deduplication
                                        // across all backups of all sync folders
                                        let nonce_slice = block_hmac.as_ref();
                                        let nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_slice[0..NONCE_SIZE as usize])
                                        .expect("failed to get nonce");

                                        // we use the same nonce both while wrapping the block key, and the block itself
                                        // this is safe because using the same nonce with 2 different keys is not nonce reuse

                                        // encrypt the chunk data using the block key
                                        let encrypted_chunk = ::sodiumoxide::crypto::secretbox::seal(&raw_chunk, &nonce, &block_key_struct);

                                        // wrap the block key with the main encryption key
                                        let wrapped_block_key = ::sodiumoxide::crypto::secretbox::seal(&block_key_raw, &nonce, &main_key);

                                        assert!(wrapped_block_key.len() == KEY_SIZE + MAC_SIZE);


                                        // prepend the key to the actual encrypted chunk data so they can be written to the file together
                                        let mut block_data = Vec::new();

                                        // first 8 bytes are the file ID, version, and reserved area
                                        let block_ver: &'static [u8; 8] = br"sdb01000";

                                        block_data.extend(block_ver.as_ref());

                                        // next 48 bytes will be the wrapped key
                                        block_data.extend(wrapped_block_key);

                                        // next 24 bytes will be the nonce/hmac
                                        block_data.extend(&block_hmac[0..NONCE_SIZE as usize]);
                                        assert!(block_data.len() == block_ver.len() + (KEY_SIZE + MAC_SIZE) + NONCE_SIZE);

                                        // remainder will be the the chunk data
                                        block_data.extend(encrypted_chunk);

                                        // set the local variable to trigger a data upload next time
                                        // we go through the loop
                                        potentially_uploaded_data = Some(block_data);

                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
                assert!(total_size == stream_length);
                debug!("calculated {} bytes of blocks, matching stream size {}", total_size, stream_length);


                let hmac_tag_size = ::sodiumoxide::crypto::auth::TAGBYTES;

                let chunklist = BufReader::new(chunks.as_slice());
                header.set_size(nb_chunk * hmac_tag_size as u64); // hmac list size
                header.set_cksum();
                ar.append(&header, chunklist).expect("failed to append chunk archive header");


                if DEBUG_STATISTICS {
                    debug!("{} chunks ({} skipped) with an average size of {} bytes.", nb_chunk, skipped_blocks, total_size / nb_chunk);

                    debug!("hmac list has {} ids <{} bytes>", chunks.len() / 32, nb_chunk * 32);
                    debug!("expected chunk size: {} bytes", expected_size);
                    debug!("smallest chunk: {} bytes.", smallest_size);
                    debug!("largest chunk: {} bytes.", largest_size);
                    debug!("standard size deviation: {} bytes.", (size_variance as f64 / nb_chunk as f64).sqrt() as u64);
                }
            } else {
                header.set_size(0); // hmac list size is zero when file has no actual data
                let chunks: Vec<u8> = Vec::new();
                let chunklist = BufReader::new(chunks.as_slice());
                header.set_cksum();
                ar.append(&header, chunklist).expect("failed to append zero length archive header");
            }
        } else if is_dir {
            // folder
            header.set_size(0); // hmac list size is zero when file has no actual data
            let chunks: Vec<u8> = Vec::new();
            let chunklist = BufReader::new(chunks.as_slice());
            header.set_cksum();
            ar.append(&header, chunklist).expect("failed to append folder to archive header");
        }
    }

    // since we're writing to a buffer in memory there shouldn't be any errors here
    let raw_session = ar.into_inner().unwrap();


    let session = SyncSession::new(SyncVersion::Version1, folder_id, session_name.to_string(), Some(archive_size), None, raw_session);
    let wrapped_session = match session.to_wrapped(main_key) {
        Ok(ws) => ws,
        Err(e) => return Err(SDError::CryptoError(e)),
    };

    let binary_data = wrapped_session.to_binary();


    debug!("finishing sync session");



    match finish_sync_session(&token, folder_id, session_name, true, &binary_data, archive_size as usize) {
        Ok(()) => {},
        Err(e) => return Err(SDError::from(e))
    };
    progress(estimated_size as u32, archive_size as u32, 0 as u32, 100.0, false);

    Ok(())
}


pub fn restore(token: &Token,
               session_name: &str,
               main_key: &Key,
               folder_id: u32,
               destination: PathBuf,
               progress: &mut FnMut(u32, u32, u32, f64, bool)) -> Result<(), SDError> {

    let folder = match get_sync_folder(token, folder_id) {
        Ok(folder) => folder,
        Err(e) => return Err(SDError::from(e))
    };

    let session_body = match read_session(token, folder_id, session_name, true) {
        Ok(session_data) => session_data,
        Err(e) => return Err(SDError::from(e))
    };

    let folder_name = &folder.folderName;

    if DEBUG_STATISTICS {
        debug!("restoring session for: {} (folder id {})", folder_name, folder_id);
    }


    let w_session = match WrappedSyncSession::from(session_body) {
        Ok(ws) => ws,
        Err(e) => return Err(e),
    };

    let session = match w_session.to_session(main_key) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let session_data: Vec<u8> = session.data;

    let mut ar = Archive::new(session_data.as_slice());

    let entry_count: u64 = ar.entries().iter().count() as u64;

    let mut failed = 0;

    let mut completed_count = 0.0;
    for item in ar.entries().unwrap() {
        let mut file_entry = item.unwrap();
        let mut full_p = PathBuf::from(&destination);

        match file_entry.path() {
            Ok(ref p) => {
                if DEBUG_STATISTICS {
                    debug!("examining {}", &p.display());
                }
                full_p.push(p);
            }
            Err(e) => {
                if DEBUG_STATISTICS {
                    debug!("not restoring invalid path: {})", e);
                }
                failed + failed + 1;
                continue // we do care about errors here, but we can't really recover from them for this item
            }
        };

        let percent_completed: f64 = (completed_count / entry_count as f64) * 100.0;

        // call out to the library user with progress
        progress(entry_count as u32, completed_count as u32, 0 as u32, percent_completed, false);

        completed_count = completed_count + 1.0;


        let entry_type = file_entry.header().entry_type();

        // process if not a directory or socket
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
                    let block_hmac_list = match ::binformat::parse_hmacs(block_hmac_bag.as_ref()) {
                        Done(_, o) => o,
                        Error(_) => return Err(SDError::CryptoError(CryptoError::BlockDecryptFailed)),
                        Incomplete(_) => panic!("should never happen")
                    };


                    for block_hmac in block_hmac_list.iter() {

                        // allow caller to tick the progress display, if one exists
                        progress(entry_count as u32, completed_count as u32, 0 as u32, percent_completed, true);

                        let mut should_retry = true;
                        let mut retries_left = 15.0;

                        let block_hmac_hex = block_hmac.to_hex();
                        println!("processing block {}", &block_hmac_hex);


                        let mut block_raw: Option<Block> = None;

                        while should_retry {
                            // allow caller to tick the progress display, if one exists
                            progress(entry_count as u32, completed_count as u32, 0 as u32, percent_completed, true);

                            let failed_count = 15.0 - retries_left;
                            let mut rng = ::rand::thread_rng();

                            // we pick a multiplier randomly to avoid a bunch of clients trying again
                            // at the same 2/4/8/16 back off time over and over if the server
                            // is overloaded or down
                            let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

                            if failed_count >= 2.0 && retries_left > 0.0 {
                                // back off significantly every time a call fails but only after the
                                // second try, the first failure could be us not including the data
                                // when we should have
                                let backoff_time = backoff_multiplier * (failed_count * failed_count);
                                let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
                                thread::sleep(delay);
                            }

                            // get block from cache if possible
                            match ::cache::read_block(&block_hmac_hex) {
                                Ok(b) => {
                                    block_raw = Some(b);
                                    debug!("cache provided block {}", &block_hmac_hex);
                                    break;
                                },
                                _ => {}
                            };

                            // get block from the server

                            match ::sdapi::read_block(&token, &block_hmac_hex) {
                                Ok(b) => {
                                    should_retry = false;
                                    debug!("server provided block {}", &block_hmac_hex);
                                    match ::cache::write_block(&b) {
                                        _ => {}
                                    };
                                    block_raw = Some(b);
                                },
                                Err(SDAPIError::RequestFailed(err)) => {
                                    retries_left = retries_left - 1.0;
                                    if retries_left <= 0.0 {
                                        // TODO: pass better error info up the call chain here rather than a failure
                                        return Err(SDError::RequestFailure(err))
                                    }
                                },
                                _ => {}
                            };
                        }

                        let block_s = block_raw.unwrap();

                        let block_wrapped: ::binformat::BinaryFormat = match ::binformat::binary_parse(&block_s.chunk_data) {
                            Done(_, o) => o,
                            Error(_) => return Err(SDError::BlockMissing),
                            Incomplete(_) => panic!("should never happen")
                        };

                        debug!("got valid binary file: {}", &block_wrapped);
                        {
                            let new_position = stream.seek(SeekFrom::Current(0)).unwrap();
                            debug!("write start {:?}", new_position);

                        }
                        let session_ver = block_wrapped.version;
                        let wrapped_block_key = block_wrapped.wrapped_key;
                        let nonce_raw = block_wrapped.nonce;
                        let wrapped_block_raw = block_wrapped.wrapped_data;

                        let nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_raw)
                            .expect("failed to get nonce");

                        let main_key_s = ::sodiumoxide::crypto::secretbox::Key::from_slice(main_key.as_ref())
                            .expect("failed to get main key struct");

                        let block_key_raw = match ::sodiumoxide::crypto::secretbox::open(&wrapped_block_key, &nonce, &main_key_s) {
                            Ok(k) => k,
                            Err(_) => return Err(SDError::CryptoError(CryptoError::BlockDecryptFailed))
                        };

                        let block_key_s = ::sodiumoxide::crypto::secretbox::Key::from_slice(&block_key_raw).expect("failed to get unwrapped block key struct");

                        let block_raw = match ::sodiumoxide::crypto::secretbox::open(&wrapped_block_raw, &nonce, &block_key_s) {
                            Ok(s) => s,
                            Err(_) => return Err(SDError::CryptoError(CryptoError::BlockDecryptFailed))
                        };
                        debug!("writing block segment of {} bytes", block_raw.len());

                        {
                            try!(stream.write_all(&block_raw));
                        }
                        {
                            let new_position = stream.seek(SeekFrom::Current(0)).unwrap();
                            debug!("new position {:?}", new_position);

                        }
                    }

                } else {
                    // empty file, just write one out with the same metadata but no body

                }
            },
            EntryType::Directory => {
                try!(fs::create_dir_all(&full_p));
            },
            EntryType::Link => {
                // hard link, may not want to try handling these yet where they exist
            },
            EntryType::Symlink => {

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

    progress(entry_count as u32, completed_count as u32, 0 as u32, 100.0, false);

    Ok(())
}