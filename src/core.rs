use std;
use std::str;
use std::ffi::{CStr};

use std::path::{Path, PathBuf};
use std::fs;
use std::net::{TcpStream};
use std::fs::File;
use std::io::{BufReader, Read, Write, Seek, SeekFrom};
use std::cmp::{min, max};

// external imports

extern crate rustc_serialize;
extern crate ssh2;
extern crate libc;
extern crate time;
extern crate sodiumoxide;
extern crate tar;
extern crate rusqlite;
extern crate libsqlite3_sys;
extern crate openssl;

extern crate walkdir;
extern crate cdc;

use self::ssh2::{Session, Sftp};
use self::tar::{Builder, Header};
use self::rusqlite::Connection;
use self::walkdir::WalkDir;
use self::cdc::*;
use self::time::Timespec;

// internal imports

use models::{Folder, Block, SyncSession};
use constants::*;
use util::*;

// internal functions

pub fn initialize(local_directory: String, unique_client_id: String) -> (PathBuf, PathBuf, PathBuf, String) {

    if !sodiumoxide::init() == true {
        panic!("Rust<sdsync_initialize>: sodium initialization failed, cannot continue");
    }

    let storage_path = Path::new(&local_directory).to_owned();

    let mut unique_client_path = Path::new(&local_directory).to_owned();
    unique_client_path.push(&unique_client_id);
    if let Err(e) = fs::create_dir_all(&unique_client_path) {
        panic!("Rust<sdsync_initialize>: failed to create local directories: {}", e);
    }

    let mut db_path = Path::new(&unique_client_path).to_owned();
    db_path.push("sdsync.sqlite3");

    match setup_tables(&db_path) {
        Ok(_) => {},
        Err(_) => panic!("Rust<sdsync_initialize>: failed to create sqlite database"),
    };

    let sodium_version = sodiumoxide::version::version_string();

    let sqlite3_version_c: &CStr = unsafe { CStr::from_ptr(libsqlite3_sys::sqlite3_libversion()) };
    let sqlite3_version: String = str::from_utf8(sqlite3_version_c.to_bytes()).unwrap().to_owned();

    let ssl_version = openssl::version::version();

    println!("Rust<sdsync_initialize>: sdsync ready <libsodium {}, sqlite {}, {}>", sodium_version, sqlite3_version, ssl_version);

    (db_path, storage_path, unique_client_path, unique_client_id)
}

pub fn setup_tables(db: &PathBuf) -> Result<(), String> {
    if let Ok(conn) = Connection::open(db) {
        conn.execute("CREATE TABLE IF NOT EXISTS folders (
                 id INTEGER PRIMARY KEY,
                 name TEXT NOT NULL UNIQUE,
                 path TEXT NOT NULL UNIQUE)",
                     &[]).unwrap();

        conn.execute("CREATE TABLE IF NOT EXISTS syncsession (
                  id INTEGER PRIMARY KEY,
                  folder_id INTEGER NOT NULL,
                  size INTEGER NOT NULL,
                  filename TEXT NOT NULL UNIQUE,
                  date DATE NOT NULL,
                  FOREIGN KEY('folder_id') REFERENCES 'folders'('id') ON DELETE CASCADE)",
                     &[]).unwrap();

        conn.execute("CREATE TABLE IF NOT EXISTS blocks (
                  id INTEGER PRIMARY KEY,
                  synced INTEGER NOT NULL,
                  refcount INTEGER NOT NULL,
                  hmac BLOB NOT NULL UNIQUE)",
                     &[]).unwrap();

        conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS block_hmac
                  on blocks (hmac) ;", &[]).unwrap();
    } else {
        return Err(format!("Rust<setup_tables>: failed to get sqlite connection to {}", &db.as_os_str().to_str().unwrap()))
    }
    Ok(())
}

pub fn get_sync_folder(db: &PathBuf,
                       folder_id: i32) -> Option<Folder> {
    if let Ok(conn) = Connection::open(&db) {
        if let Ok(folder) = conn.query_row("SELECT * FROM folders WHERE id = ?", &[&folder_id], |row| {
            let s_id = row.get(0);
            let name: String = row.get(1);
            let path: String = row.get(2);
            Folder {
                id: s_id,
                name: name,
                path: path,
            }
        }) {
            return Some(folder);
        };
    }
    return None
}

pub fn add_sync_folder(db: PathBuf,
                       id: i32,
                       name: &str,
                       path: &str) -> Result<(), String> {
    if let Ok(conn) = Connection::open(&db) {
        if let Ok(_) = conn.execute("INSERT INTO folders (id, name, path) VALUES ($1, $2, $3)", &[&id, &name, &path]) {
            return Ok(())
        } else {
            return Err(format!("Rust<sdsync_add_sync_folder>: failed to store folder"))
        }
    };
    Ok(())
}

pub fn sync_folders(db: &PathBuf) -> Result<Vec<Folder>, String> {
    let mut folder_list: Vec<Folder> = Vec::new();
    if let Ok(conn) = Connection::open(db) {
        let mut stmt = conn.prepare("SELECT id, name, path FROM folders").unwrap();
        let folder_iter = stmt.query_map(&[], |row| {
            let name: String = row.get(1);
            let path: String = row.get(2);
            let s_id = row.get(0);
            Folder {
                id: s_id,
                name: name,
                path: path,
            }
        }).unwrap();


        for result in folder_iter {
            if let Ok(folder) = result {
                folder_list.push(folder);
            } else {
                return Err(format!("Rust<sync_folders>: failed to unwrap folder result!"))
            }
        }
    } else {
        return Err(format!("Rust<sync_folders>: failed to get sqlite connection"))
    }
    return Ok(folder_list)
}

pub fn sync_sessions(db: &PathBuf, folder_id: i32) -> Result<Vec<SyncSession>, String> {
    let mut session_list: Vec<SyncSession> = Vec::new();
    if let Ok(conn) = Connection::open(db) {
        let mut stmt = conn.prepare("SELECT * FROM syncsession WHERE id = ?").unwrap();
        let session_iter = stmt.query_map(&[&folder_id], |row| {
            let s_id = row.get(0);
            let folder_id = row.get(1);
            let filename: String = row.get(3);
            let date: Timespec = row.get(4);

            SyncSession {
                id: s_id,
                folder_id: folder_id,
                size: 0,
                filename: filename,
                date: date
            }
        }).unwrap();

        for result in session_iter {
            if let Ok(session) = result {
                session_list.push(session);
            } else {
                return Err(format!("Rust<sync_sessions>: failed to unwrap session result!"))
            }
        }
    } else {
        return Err(format!("Rust<sync_sessions>: failed to get sqlite connection"))
    }
    Ok(session_list)
}


pub fn add_block(db: &PathBuf,
                 sftp_session: &Sftp,
                 unique_client_id: &str,
                 hmac: &[u8],
                 key: &[u8],
                 chunk_data: &[u8]) -> bool {
    // we always write the chunk to disk first, before recording it in the block table. This ensures
    // that we never end up with a record of a chunk that we don't have, which is the safest choice
    // for protecting user data. It comes at a slight cost, the risk that we'll end up "leaking" a
    // block with no record of it if the sync ever cancels or fails right after writing the chunk to
    // disk. We can manage that by enumerating all blocks and checking to see if they're referenced
    // by an archive and delete any that aren't.

    // At some point we might be able to make this more transactional, so that if one or the other
    // fails, the other is rolled back for consistency

    // prepend the key to the actual encrypted chunk data so they can be written to the file together
    let mut to_write = Vec::new();

    // bytes 0-47 will be the key
    to_write.extend(key);

    // byte 48+ will be the the chunk data
    to_write.extend(chunk_data);

    // we store chunks inside a sharded directory structure consisting of 15 separate
    // 0-9 and a-f (lowercase) directories, each chunk will be placed in one of the directories
    // depending on the first hex character of the HMAC value

    let chunk_path = block_directory(&unique_client_id, &hmac);

    // remote chunk writing
    let mut file = match sftp_session.create(&chunk_path) {
        Ok(file) => file,
        Err(e) => { println!("Rust<sdsync_add_block>: sftp file create failed: {}", e); return false },
    };

    match file.write_all(&to_write) {
        Err(e) => println!("Rust<sdsync_add_block>: sftp write failed: {}", e),
        _ => {}
    };

    // now that we know the chunk is safely written to disk somewhere, we can record it in the table
    if let Ok(conn) = Connection::open(&db) {
        match conn.execute("INSERT INTO blocks (synced, refcount, hmac) VALUES ($1, $2, $3)", &[&1, &1, &hmac]) {
            Err(e) => println!("Rust<sdsync_add_block>: failed to store block {}", e),
            _ => return true
        };
    }
    return false
}

pub fn have_stored_block(db: &PathBuf,
                         hmac: &[u8]) -> bool {
    if let Ok(conn) = Connection::open(db) {
        if let Ok(_) = conn.query_row("SELECT * FROM blocks WHERE hmac = ?", &[&hmac], |row| {
            let s_id = row.get(0);
            Block {
                id: s_id,
                synced: row.get(1),
                refcount: row.get(2),
                hmac: row.get(3),
            }
        }) {
            return true
        };
    } else {
        println!("Rust<have_stored_block>: failed to get sqlite connection");
    }
    return false
}















pub fn create_archive(main_key_array: [u8; 32],
                      hmac_key_array: [u8; 32],
                      ssh_username: &str,
                      ssh_password: &str,
                      safedrive_sftp_client_ip: &str,
                      safedrive_sftp_client_port: u16,
                      unique_client_id: &str,
                      db: PathBuf,
                      folder_id: i32) -> Result<(), String> {

    let archive_file = Vec::new();

    if let Some(folder) = get_sync_folder(&db, folder_id) {
        if DEBUG_STATISTICS {
            println!("Rust<sdsync_create_archive>: creating archive for: {}", folder_id);
        }

        let mut ar = Builder::new(archive_file);
        let mut archive_size: i64 = 0;

        // start an SSH session to upload files later
        let ip: &str = safedrive_sftp_client_ip;

        let tcp = TcpStream::connect((ip, safedrive_sftp_client_port)).unwrap();
        let mut ssh_session = Session::new().unwrap();
        ssh_session.set_compress(true);

        ssh_session.handshake(&tcp).unwrap();

        ssh_session.userauth_password(ssh_username, ssh_password).unwrap();
        assert!(ssh_session.authenticated());
        let sftp_session = ssh_session.sftp().unwrap();

        // ensure remote directories exist
        match create_directories(&sftp_session, &unique_client_id) {
            true => {},
            false => return Err("Rust<sdsync_create_archive>: failed to create remote directories".to_string())
        };
        let entry_count = WalkDir::new(&folder.path).into_iter().count() as u64;

        let mut failed = 0;

        let mut progress = 0.0;
        for item in WalkDir::new(&folder.path).into_iter().filter_map(|e| e.ok()) {
            if DEBUG_STATISTICS {
                println!("Rust<sdsync_create_archive>: examining {}", item.path().display());
            }
            progress = progress + 1.0;
            let percent_completed: f64 = (progress / entry_count as f64) * 100.0;
            let p = item.path();
            let p_relative = p.strip_prefix(&folder.path).expect("failed to unwrap relative path");

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


                        let main_key_slice: &[u8] = &main_key_array;
                        let main_key = sodiumoxide::crypto::secretbox::Key::from_slice(main_key_slice)
                            .expect("failed to get main key struct");

                        // calculate hmac of the block
                        let hmac_key_slice: &[u8] = &hmac_key_array;
                        let hmac_key = sodiumoxide::crypto::auth::Key::from_slice(hmac_key_slice)
                            .expect("failed to get hmac key struct");

                        let block_hmac = sodiumoxide::crypto::auth::authenticate(raw_chunk, &hmac_key);
                        chunks.extend_from_slice(block_hmac.as_ref());


                        // check if we've stored this chunk before

                        if !have_stored_block(&db, &block_hmac.as_ref()) {
                            // generate a new chunk key
                            let size = sodiumoxide::crypto::secretbox::KEYBYTES;
                            let block_key_raw = sodiumoxide::randombytes::randombytes(size);
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
                            let nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_slice[0..NONCE_SIZE as usize])
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
                    let chunklist = BufReader::new(chunks.as_slice());
                    header.set_size(nb_chunk * HMAC_SIZE as u64); // hmac list size
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
                }
                    else {
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

        let main_key_slice: &[u8] = &main_key_array;
        let main_key = sodiumoxide::crypto::secretbox::Key::from_slice(main_key_slice)
            .expect("Rust<sdsync_create_archive>: failed to get main key struct");

        // generate a new archive key
        let size = sodiumoxide::crypto::secretbox::KEYBYTES;
        let archive_key_raw = sodiumoxide::randombytes::randombytes(size);
        let archive_key_struct = sodiumoxide::crypto::secretbox::Key::from_slice(&archive_key_raw)
            .expect("Rust<sdsync_create_archive>: failed to get archive key struct");

        // We use a random nonce here because we don't need to know what it is in advance, unlike blocks
        let nonce_raw = sodiumoxide::randombytes::randombytes(size);
        let nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_raw[0..24])
            .expect("Rust<sdsync_create_archive>: failed to get nonce");

        // we use the same nonce both while wrapping the archive key, and the archive itself
        // this is safe because using the same nonce with 2 different keys is not nonce reuse

        // encrypt the archive data using the archive key
        let encrypted_archive = sodiumoxide::crypto::secretbox::seal(&raw_archive, &nonce, &archive_key_struct);

        // wrap the archive key with the main encryption key
        let wrapped_archive_key = sodiumoxide::crypto::secretbox::seal(&archive_key_raw, &nonce, &main_key);

        let mut complete_archive = Vec::new();

        // bytes 0-47 will be the wrapped archive key
        complete_archive.extend(wrapped_archive_key);

        // bytes 48-72 will be the nonce
        complete_archive.extend(nonce_raw);

        // byte 73+ will be the encrypted archive data
        complete_archive.extend(encrypted_archive);


        let t = time::get_time();

        let archive_name = Uuid::new_v4().hyphenated().to_string() + ".sdsyncv1";
        let mut archive_path = archive_directory(&unique_client_id);
        archive_path.push(&archive_name);
        if DEBUG_STATISTICS {
            println!("Rust<sdsync_create_archive>: uploading archive to: {}", archive_path.display());
        }

        let mut file = match sftp_session.create(&archive_path) {
            Ok(file) => file,
            Err(e) => { return Err(format!("Rust<sdsync_create_archive>: archive create: {}", e)) },
        };

        match file.write_all(&complete_archive) {
            Err(e) => { return Err(format!("Rust<sdsync_create_archive>: archive write failed: {}", e)) },
            _ => {}
        };

        // now that we know the archive is safely written to disk somewhere, we can record it in the table
        if let Ok(conn) = Connection::open(&db) {
            match conn.execute("INSERT INTO syncsession \
                            (folder_id, size, filename, date) \
                            VALUES ($1, $2, $3, $4)",
                               &[&folder_id, &archive_size, &archive_name, &t]) {
                Err(e) => return Err(format!("Rust<sdsync_create_archive>:  failed to store archive record {}", e)),
                _ => {}
            };
        }
    }
    Ok(())
}


















pub fn create_directories(sftp_session: &Sftp, unique_client_id: &str) -> bool {
    let mut storage_dir = Path::new("/storage/").to_owned();
    if let Err(_) = sftp_session.mkdir(&storage_dir, DEFAULT_PERMISSIONS) {
        //
    };

    storage_dir.push(&unique_client_id);
    if let Err(_) = sftp_session.mkdir(&storage_dir, DEFAULT_PERMISSIONS) {
        //
    };

    storage_dir.push(&Path::new("archives"));
    if let Err(_) = sftp_session.mkdir(&storage_dir, DEFAULT_PERMISSIONS) {
        //
    };
    storage_dir.pop();

    storage_dir.push("blocks");
    if let Err(_) = sftp_session.mkdir(&storage_dir, DEFAULT_PERMISSIONS) {
        //
    };

    // create individual shard directories
    for subdirectory in BLOCK_DIRECTORIES {
        let mut bdir = Path::new(&storage_dir).to_owned();
        bdir.push(&Path::new(&subdirectory));
        if let Err(_) = sftp_session.mkdir(&bdir, DEFAULT_PERMISSIONS) {
            //
        };
    }
    true
}