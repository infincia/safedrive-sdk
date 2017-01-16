use std;
use std::str;

use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::cmp::{min, max};
use std::{thread, time};

// external imports

extern crate rustc_serialize;
extern crate libc;
extern crate sodiumoxide;
extern crate tar;
extern crate rand;

#[cfg(target_os = "linux")]
extern crate openssl;

extern crate walkdir;
extern crate cdc;

use self::rustc_serialize::hex::{ToHex};

use self::rand::distributions::{IndependentSample, Range};

use self::tar::{Builder, Header};
use self::walkdir::WalkDir;
use self::cdc::*;

// internal imports

use models::*;
use constants::*;
use sdapi::*;
use keys::*;
use error::{CryptoError, SDAPIError};

// internal functions

pub fn initialize<S, T>(local_directory: S, unique_client_id: T) -> (PathBuf, String) where S: Into<String>, T: Into<String>{

    let ldir = local_directory.into();
    let uid = unique_client_id.into();

    if !sodiumoxide::init() == true {
        panic!("Rust<sdsync_initialize>: sodium initialization failed, cannot continue");
    }

    let storage_path = Path::new(&ldir).to_owned();

    if let Err(e) = fs::create_dir_all(&storage_path) {
        panic!("Rust<sdsync_initialize>: failed to create local directories: {}", e);
    }


    let sodium_version = sodiumoxide::version::version_string();
    debug!("libsodium {}", sodium_version);

    #[cfg(target_os = "linux")]
    let ssl_version = openssl::version::version();
    #[cfg(target_os = "linux")]
    debug!("{}>", ssl_version);

    debug!("ready");

    (storage_path, uid)
}

pub fn login(username: &str,
             password:  &str) -> Result<(Token, AccountStatus, UniqueClientID), String> {

    match register_client(username, password) {
        Ok((t, ucid)) => {
            match account_status(&t) {
                Ok(s) => return Ok((t, s, ucid)),
                Err(e) => return Err(format!("failed to register client: {:?}", e))
            }
        },
        Err(e) => return Err(format!("failed to register client: {:?}", e))
    }
}

pub fn load_keys(token: &Token, recovery_phrase: Option<String>, store_recovery_key: &Fn(&str)) -> Result<Keyset, CryptoError> {
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
        Err(_) => return Err(CryptoError::KeyGenerationFailed)
    };


    if let Ok(real_wrapped_keyset) = account_key(token, &new_wrapped_keyset) {
        // now we check to see if the keys returned by the server match the existing phrase or not

        // if we were given an existing phrase try it, otherwise try the new one
        debug!("got key set back from server, checking");

        if let Some(p) = recovery_phrase {
            match real_wrapped_keyset.to_keyset(&p) {
                Ok(ks) => {
                    return Ok(ks)
                },
                Err(e) => {
                    debug!("failed to decrypt keys: {:?}", e);

                    return Err(CryptoError::DecryptFailed)
                }
            };
        } else if let Some(p) = new_wrapped_keyset.recovery {
            match real_wrapped_keyset.to_keyset(&p) {
                Ok(ks) => {
                    // a new keyset was generated so we must return the phrase to the caller so it
                    // can be stored and displayed
                    store_recovery_key(&p);
                    return Ok(ks)
                },
                Err(e) => {
                    debug!("failed to decrypt keys: {:?}", e);

                    return Err(CryptoError::DecryptFailed)
                }
            };
        }
    };
    return Err(CryptoError::RetrieveFailed)
}

#[allow(unused_variables)]
pub fn get_sync_folder(token: &Token,
                       folder_id: u32) -> Result<RegisteredFolder, String> {
    let folders = match read_folders(token) {
        Ok(folders) => folders,
        Err(e) => return Err(format!("Rust<get_sync_folder>: getting folder failed: {:?}", e))
    };
    for folder in folders {
        if folder.id == folder_id {
            return Ok(folder)
        }
    }
    return Err(format!("Rust<get_sync_folder>: getting folder failed"))
}

pub fn add_sync_folder(token: &Token,
                       name: &str,
                       path: &str) -> Result<u32, String> {
    match create_folder(token, path, name, true) {
        Ok(folder_id) => Ok(folder_id),
        Err(e) => return Err(format!("Rust<add_sync_folder>: creating folder failed: {:?}", e))
    }
}

pub fn remove_sync_folder(token: &Token,
                          folder_id: u32) -> Result<(), String> {
    match delete_folder(token, folder_id) {
        Ok(()) => Ok(()),
        Err(e) => return Err(format!("Rust<remove_sync_folder>: deleting folder failed: {:?}", e))
    }
}

pub fn get_sync_folders(token: &Token) -> Result<Vec<RegisteredFolder>, String> {
    match read_folders(token) {
        Ok(folders) => Ok(folders),
        Err(e) => return Err(format!("Rust<get_sync_folders>: getting folder failed: {:?}", e))
    }
}

pub fn get_sync_sessions(token: &Token) -> Result<Vec<SyncSession>, String> {
    let m = match read_sessions(token) {
        Ok(sessions) => sessions,
        Err(e) => return Err(format!("Rust<get_sync_sessions>: getting sessions failed: {:?}", e))
    };
    let mut v: Vec<SyncSession> = Vec::new();

    for (folder_id, sessions) in &m {
        for session in sessions {
            let mut s = session.clone();
            s.folder_id = *folder_id;
            v.push(s);
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
            progress: &mut FnMut(u32, u32, f64, bool)) -> Result<(), String> {

    let folder = match get_sync_folder(token, folder_id) {
        Ok(folder) => folder,
        Err(e) => return Err(format!("Rust<sdsync_create_archive>: failed to get sync folder info from server: {:?}", e))
    };

    match register_sync_session(token, folder_id, session_name, true) {
        Ok(()) => {},
        Err(e) => return Err(format!("Rust<sdsync_create_archive>: registering sync session failed: {:?}", e))
    };

    let folder_path = PathBuf::from(&folder.folderPath);
    let folder_name = &folder.folderName;


    let archive_file = Vec::new();

    if DEBUG_STATISTICS {
        debug!("creating archive for: {} (folder id {})", folder_name, folder_id);
    }
    let key_size = sodiumoxide::crypto::secretbox::KEYBYTES;
    let nonce_size = sodiumoxide::crypto::secretbox::NONCEBYTES;
    let mac_size = sodiumoxide::crypto::secretbox::MACBYTES;

    let mut ar = Builder::new(archive_file);
    let mut archive_size: i64 = 0;

    let entry_count = WalkDir::new(&folder_path).into_iter().count() as u64;

    let mut failed = 0;

    let mut completed_count = 0.0;
    for item in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
        if DEBUG_STATISTICS {
            debug!("examining {}", item.path().display());
        }
        let percent_completed: f64 = (completed_count / entry_count as f64) * 100.0;

        // call out to the library user with progress
        progress(entry_count as u32, completed_count as u32, percent_completed, false);

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
                let byte_iter = reader.bytes().map(|b| b.expect("Rust<sdsync_create_archive>: failed to unwrap block data"));

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
                let expected_size = 1 << 13;
                let mut size_variance = 0;
                let mut chunks: Vec<u8> = Vec::new();
                for chunk in chunk_iter {
                    // allow caller to tick the progress display, if one exists
                    progress(entry_count as u32, completed_count as u32, percent_completed, true);

                    nb_chunk += 1;
                    total_size += chunk.size;

                    // truncating here for compatibility with the sqlite model
                    // it is safe because it is virtually impossible that total archive
                    // size will even reach i64 size,(8192 petabytes) and u64 is 2x larger still
                    archive_size += total_size as i64;


                    smallest_size = min(smallest_size, chunk.size);
                    largest_size = max(largest_size, chunk.size);
                    size_variance += (chunk.size as i64 - expected_size as i64).pow(2);
                    f_chunk.seek(SeekFrom::Start(chunk.index)).expect("failed to seek chunk reader");

                    let mut buffer = BufReader::new(&f_chunk).take(chunk.size);
                    let mut data: Vec<u8> = Vec::with_capacity(100000); //expected size of largest block

                    if let Err(e) = buffer.read_to_end(&mut data) {
                        return Err(format!("could not read from file: {}", e))
                    }

                    let raw_chunk = data.as_slice();


                    let main_key = sodiumoxide::crypto::secretbox::Key::from_slice(main_key.as_ref())
                        .expect("failed to get main key struct");

                    // calculate hmac of the block
                    let hmac_key = sodiumoxide::crypto::auth::Key::from_slice(hmac_key.as_ref())
                        .expect("failed to get hmac key struct");

                    let block_hmac = sodiumoxide::crypto::auth::authenticate(raw_chunk, &hmac_key);
                    chunks.extend_from_slice(block_hmac.as_ref());

                    let mut should_retry = true;
                    let mut retries_left = 15.0;
                    let mut potentially_uploaded_data: Option<Vec<u8>> = None;

                    // generate a new chunk key once in case we need it later. this is cheap to do
                    let block_key_raw = sodiumoxide::randombytes::randombytes(key_size);

                    while should_retry {
                        // allow caller to tick the progress display, if one exists
                        progress(entry_count as u32, completed_count as u32, percent_completed, true);

                        if retries_left <= 0.0 {
                            return Err(format!("could not sync: {}", &folder_path.to_str().unwrap()))
                        }
                        let failed_count = 15.0 - retries_left;
                        let mut rng = rand::thread_rng();

                        // we pick a multiplier randomly to avoid a bunch of clients trying again
                        // at the same 2/4/8/16 backoff times time over and over if the server
                        // is overloaded or down
                        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

                        if failed_count >= 2.0 {
                            // back off significantly every time a call fails but only after the
                            // second try, the first failure could be us not including the data
                            // when we should have
                            let backoff_time = backoff_multiplier * (failed_count * failed_count);
                            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
                            thread::sleep(delay);
                        }

                        // tell the server to mark this block without the data first, if that fails we try uploading

                        match write_block(&token, session_name, block_hmac.as_ref().to_hex(), &potentially_uploaded_data) {
                            Ok(()) => {
                                skipped_blocks = skipped_blocks + 1;
                                should_retry = false
                            },
                            Err(SDAPIError::RequestFailed) => {
                                retries_left = retries_left - 1.0;
                            },
                            Err(SDAPIError::RetryUpload) => {
                                retries_left = retries_left - 1.0;

                                match potentially_uploaded_data {
                                    Some(_) => {},
                                    None => {

                                        let block_key_struct = sodiumoxide::crypto::secretbox::Key::from_slice(&block_key_raw)
                                        .expect("failed to get block key struct");

                                        // We use the first 24 bytes of the block hmac value as nonce for wrapping
                                        // the block key and encrypting the block itself.
                                        //
                                        // This is cryptographically safe but still deterministic: encrypting
                                        // the same block twice with a specific key will always produce the same
                                        // output block, which is critical for versioning and deduplication
                                        // across all backups of all sync folders
                                        let nonce_slice = block_hmac.as_ref();
                                        let nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_slice[0..nonce_size as usize])
                                        .expect("Rust<sdsync_create_archive>: failed to get nonce");

                                        // we use the same nonce both while wrapping the block key, and the block itself
                                        // this is safe because using the same nonce with 2 different keys is not nonce reuse

                                        // encrypt the chunk data using the block key
                                        let encrypted_chunk = sodiumoxide::crypto::secretbox::seal(&raw_chunk, &nonce, &block_key_struct);

                                        // wrap the block key with the main encryption key
                                        let wrapped_block_key = sodiumoxide::crypto::secretbox::seal(&block_key_raw, &nonce, &main_key);

                                        assert!(wrapped_block_key.len() == key_size + mac_size);


                                        // prepend the key to the actual encrypted chunk data so they can be written to the file together
                                        let mut block_data = Vec::new();

                                        // bytes 0-47 will be the wrapped key
                                        block_data.extend(wrapped_block_key);

                                        // bytes 48-71 will be the nonce/hmac
                                        block_data.extend(&block_hmac[0..nonce_size as usize]);
                                        assert!(block_data.len() == key_size + mac_size + nonce_size);

                                        // byte 72+ will be the the chunk data
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
                let hmac_tag_size = sodiumoxide::crypto::auth::TAGBYTES;

                let chunklist = BufReader::new(chunks.as_slice());
                header.set_size(nb_chunk * hmac_tag_size as u64); // hmac list size
                header.set_cksum();
                ar.append(&header, chunklist).expect("Rust<sdsync_create_archive>: failed to append chunk archive header");

                if nb_chunk != skipped_blocks && DEBUG_STATISTICS {
                    debug!("{} chunks ({} skipped) with an average size of {} bytes.", nb_chunk, skipped_blocks, total_size / nb_chunk);
                }
                if DEBUG_STATISTICS {
                    debug!("hmac list has {} ids <{}>", chunks.len() / 32, nb_chunk * 32);
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
                ar.append(&header, chunklist).expect("Rust<sdsync_create_archive>: failed to append zero length archive header");
            }
        } else if is_dir {
            // folder
            header.set_size(0); // hmac list size is zero when file has no actual data
            let chunks: Vec<u8> = Vec::new();
            let chunklist = BufReader::new(chunks.as_slice());
            header.set_cksum();
            ar.append(&header, chunklist).expect("Rust<sdsync_create_archive>: failed to append folder to archive header");
        }
    }

    if let Err(e) = ar.finish() {
        return Err(format!("error finalizing archive: {}", e))
    }

    let raw_archive = &ar.into_inner().unwrap();

    // get the main key

    let main_key = sodiumoxide::crypto::secretbox::Key::from_slice(main_key.as_ref())
        .expect("Rust<sdsync_create_archive>: failed to get main key struct");

    // generate a new archive key
    let key_size = sodiumoxide::crypto::secretbox::KEYBYTES;
    let nonce_size = sodiumoxide::crypto::secretbox::NONCEBYTES;
    let mac_size = sodiumoxide::crypto::secretbox::MACBYTES;

    let archive_key_raw = sodiumoxide::randombytes::randombytes(key_size);
    let archive_key_struct = sodiumoxide::crypto::secretbox::Key::from_slice(&archive_key_raw)
        .expect("Rust<sdsync_create_archive>: failed to get archive key struct");

    // We use a random nonce here because we don't need to know what it is in advance, unlike blocks
    let nonce_raw = sodiumoxide::randombytes::randombytes(nonce_size);
    let nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_raw[0..24])
        .expect("Rust<sdsync_create_archive>: failed to get nonce");

    // we use the same nonce both while wrapping the archive key, and the archive itself
    // this is safe because using the same nonce with 2 different keys is not nonce reuse

    // encrypt the archive data using the archive key
    let encrypted_archive = sodiumoxide::crypto::secretbox::seal(&raw_archive, &nonce, &archive_key_struct);

    // wrap the archive key with the main encryption key
    let wrapped_archive_key = sodiumoxide::crypto::secretbox::seal(&archive_key_raw, &nonce, &main_key);
    assert!(wrapped_archive_key.len() == key_size + mac_size);

    let mut complete_archive = Vec::new();

    // bytes 0-47 will be the wrapped archive key
    complete_archive.extend(wrapped_archive_key);

    // bytes 48-71 will be the nonce
    complete_archive.extend(nonce_raw);

    // byte 72+ will be the encrypted archive data
    complete_archive.extend(encrypted_archive);

    let mut archive_name: String = String::new();
    archive_name += session_name;
    archive_name += ".sdsyncv1";


    debug!("finishing sync session");



    match finish_sync_session(&token, folder_id, archive_name, true, &complete_archive, archive_size as usize) {
        Ok(()) => {},
        Err(e) => return Err(format!("Rust<sdsync_create_archive>: finishing session failed: {:?}", e))
    };
    progress(entry_count as u32, completed_count as u32, 100.0, false);

    Ok(())
}