use std;
use std::str;

use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::cmp::{min, max};

// external imports

extern crate rustc_serialize;
extern crate libc;
extern crate time;
extern crate sodiumoxide;
extern crate tar;

#[cfg(unix)]
extern crate openssl;

extern crate walkdir;
extern crate cdc;

use self::rustc_serialize::hex::{ToHex};


use self::tar::{Builder, Header};
use self::walkdir::WalkDir;
use self::cdc::*;

// internal imports

use models::*;
use constants::*;
use util::*;
use sdapi::*;
use keys::*;
use error::CryptoError;

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
    println!("Rust<sdsync_initialize>: libsodium {}", sodium_version);

    #[cfg(unix)]
    let ssl_version = openssl::version::version();
    #[cfg(unix)]
    println!("Rust<sdsync_initialize>: {}>", ssl_version);

    println!("Rust<sdsync_initialize>: ready");

    (storage_path, uid)
}

pub fn login(username: &str,
             password:  &str) -> Result<(Token, AccountStatus, UniqueClientID), String> {

    match client_register(username, password) {
        Ok((t, ucid)) => {
            match account_status(&t) {
                Ok(s) => return Ok((t, s, ucid)),
                Err(e) => return Err(format!("failed to register client: {:?}", e))
            }
        },
        Err(e) => return Err(format!("failed to register client: {:?}", e))
    }
}

pub fn load_keys(token: &Token, recovery_phrase: Option<String>, store_recovery_key: &Fn(&str)) -> Result<(Key, Key, Key), CryptoError> {
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
    let (new_phrase, master_key_wrapped, main_key_wrapped, hmac_key_wrapped) = match generate_keyset() {
        Ok((p, mas, main, hmac)) => (p, mas, main, hmac),
        Err(_) => return Err(CryptoError::GenerateFailed)
    };


    if let Ok((real_master_wrapped, real_main_wrapped, real_hmac_wrapped)) = account_key(token, master_key_wrapped.to_hex(), main_key_wrapped.to_hex(), hmac_key_wrapped.to_hex()) {
        // now we check to see if the keys returned by the server match the existing phrase or not

        // if we were given an existing phrase try it, otherwise try the new one
        println!("Rust<load_keys>: got key set back from server, checking");

        let phrase_to_check = match recovery_phrase {
            Some(p) => p,
            None => new_phrase
        };

        let wrapped_master_key = try!(WrappedKey::from_hex(real_master_wrapped, KeyType::KeyTypeMaster));
        let wrapped_main_key = try!(WrappedKey::from_hex(real_main_wrapped, KeyType::KeyTypeMain));
        let wrapped_hmac_key = try!(WrappedKey::from_hex(real_hmac_wrapped, KeyType::KeyTypeHMAC));
        println!("Rust<load_keys>: loaded wrapped keys");

        match decrypt_keyset(&phrase_to_check, wrapped_master_key, wrapped_main_key, wrapped_hmac_key) {
            Ok((real_phrase, real_master_key, real_main_key, real_hmac_key)) => {
                if real_phrase != phrase_to_check {
                    // we got a working recovery phrase that was NOT passed to the library as an
                    // argument, so it's a new phrase and we must return it to the caller so it
                    // can be stored and displayed
                    store_recovery_key(&phrase_to_check);
                }
                return Ok((real_master_key, real_main_key, real_hmac_key))
            },
            Err(e) => {
                println!("Rust<load_keys>: failed to decrypt keys: {:?}", e);

                return Err(CryptoError::DecryptFailed)
            }
        };
    };
    return Err(CryptoError::RetrieveFailed)
}

pub fn get_sync_folder(folder_id: i32) -> Option<Folder> {

    return None
}

pub fn add_sync_folder(name: &str,
                       path: &str) -> Result<(), String> {

    Ok(())
}

pub fn sync_folders() -> Result<Vec<Folder>, String> {
    let mut folder_list: Vec<Folder> = Vec::new();

    return Ok(folder_list)
}

pub fn sync_sessions(folder_id: i32) -> Result<Vec<SyncSession>, String> {
    let mut session_list: Vec<SyncSession> = Vec::new();

    Ok(session_list)
}

pub fn create_archive(token: &Token,
                      session_name: &str,
                      main_key: &Key,
                      hmac_key: &Key,
                      folder_id: i32,
                      folder_path: PathBuf,
                      progress: &mut FnMut(f64)) -> Result<(), String> {

    let archive_file = Vec::new();

    if true {
        if DEBUG_STATISTICS {
            println!("Rust<sdsync_create_archive>: creating archive for: {}", folder_id);
        }

        let mut ar = Builder::new(archive_file);
        let mut archive_size: i64 = 0;

        let entry_count = WalkDir::new(&folder_path).into_iter().count() as u64;

        let mut failed = 0;

        let mut completed_count = 0.0;
        for item in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
            if DEBUG_STATISTICS {
                println!("Rust<sdsync_create_archive>: examining {}", item.path().display());
            }
            let percent_completed: f64 = (completed_count / entry_count as f64) * 100.0;

            // call out to the library user with progress
            progress(percent_completed);

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
                    println!("Rust<sdsync_create_archive>: NOTICE - not adding invalid path: '{}' (reason: {})", p_relative.display(), err);
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
                    let separator_iter = SeparatorIter::new(byte_iter);
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
                            return Err(format!("Rust<sdsync_create_archive>: could not read from file: {}", e))
                        }

                        let raw_chunk = data.as_slice();


                        let main_key = sodiumoxide::crypto::secretbox::Key::from_slice(main_key.as_ref())
                            .expect("failed to get main key struct");

                        // calculate hmac of the block
                        let hmac_key = sodiumoxide::crypto::auth::Key::from_slice(hmac_key.as_ref())
                            .expect("failed to get hmac key struct");

                        let block_hmac = sodiumoxide::crypto::auth::authenticate(raw_chunk, &hmac_key);
                        chunks.extend_from_slice(block_hmac.as_ref());


                        // check if we've stored this chunk before

                        if !have_stored_block(&db, &block_hmac.as_ref()) {
                            // generate a new chunk key
                            let key_size = sodiumoxide::crypto::secretbox::KEYBYTES;
                            let nonce_size = sodiumoxide::crypto::secretbox::NONCEBYTES;

                            let block_key_raw = sodiumoxide::randombytes::randombytes(key_size);
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

                            // pass the hmac, wrapped key, and encrypted block out to storage code
                            if !add_block(&db, &sftp_session, &unique_client_id, &block_hmac.as_ref(), &wrapped_block_key, &encrypted_chunk) {
                                return Err(format!("Rust<sdsync_create_archive>: failed to add block!!!"))
                            }
                        } else {
                            skipped_blocks = skipped_blocks + 1;
                        }
                    }
                    let hmac_tag_size = sodiumoxide::crypto::auth::TAGBYTES;

                    let chunklist = BufReader::new(chunks.as_slice());
                    header.set_size(nb_chunk * hmac_tag_size as u64); // hmac list size
                    header.set_cksum();
                    ar.append(&header, chunklist).expect("Rust<sdsync_create_archive>: failed to append chunk archive header");

                    if nb_chunk != skipped_blocks && DEBUG_STATISTICS {
                        println!("Rust<sdsync_create_archive>: {} chunks ({} skipped) with an average size of {} bytes.", nb_chunk, skipped_blocks, total_size / nb_chunk);
                    }
                    if DEBUG_STATISTICS {
                        println!("Rust<sdsync_create_archive>: hmac list has {} ids <{}>", chunks.len() / 32, nb_chunk * 32);
                        println!("Rust<sdsync_create_archive>: Expected chunk size: {} bytes", expected_size);
                        println!("Rust<sdsync_create_archive>: Smallest chunk: {} bytes.", smallest_size);
                        println!("Rust<sdsync_create_archive>: Largest chunk: {} bytes.", largest_size);
                        println!("Rust<sdsync_create_archive>: Standard size deviation: {} bytes.", (size_variance as f64 / nb_chunk as f64).sqrt() as u64);
                    }
                } else {
                    header.set_size(0); // hmac list size is zero when file has no actual data
                    let chunks: Vec<u8> = Vec::new();
                    let chunklist = BufReader::new(chunks.as_slice());
                    header.set_cksum();
                    ar.append(&header, chunklist).expect("Rust<sdsync_create_archive>: failed to append zero length archive header");
                }
            }
                else if is_dir {
                    // folder
                    header.set_size(0); // hmac list size is zero when file has no actual data
                    let chunks: Vec<u8> = Vec::new();
                    let chunklist = BufReader::new(chunks.as_slice());
                    header.set_cksum();
                    ar.append(&header, chunklist).expect("Rust<sdsync_create_archive>: failed to append folder to archive header");
                }
        }

        if let Err(e) = ar.finish() {
            return Err(format!("Rust<sdsync_create_archive>: error finalizing archive: {}", e))
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

        if DEBUG_STATISTICS {
            println!("Rust<sdsync_create_archive>: finishing sync session");
        }

    }
    progress(100.0);

    Ok(())
}