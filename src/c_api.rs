use std;
use std::ffi::{CStr, CString};
use std::str;
use std::slice;
use std::path::{Path};
use std::mem;
use std::mem::forget;

use std::u64;
use std::str::FromStr;

// external imports

extern crate libc;
extern crate sodiumoxide;

//internal imports

use ::models::Context;

use ::core::initialize;
use ::core::add_sync_folder;
use ::core::sync_folders;
use ::core::sync_sessions;
use ::core::create_archive;

// exports



#[derive(Debug)]
#[repr(C)]
pub struct CContext(Context);

#[derive(Debug)]
#[repr(C)]
pub struct CFolder {
    pub id: i64,
    pub name: *mut std::os::raw::c_char,
    pub path: *mut std::os::raw::c_char,
}

#[derive(Debug)]
#[repr(C)]
pub struct CSyncSession {
    pub id: i64,
    pub filename: *const std::os::raw::c_char,
    pub size: i64,
    pub date: f64,
    pub folder_id: i32,
}


/// Initialize the library, must be called before any other function
///
/// Will assert non-null on each parameter
///
/// Parameters:
///
///     db_path: a stack-allocated, NULL-terminated string representing the location for local database and block storage
///
///     unique_client_id: a stack-allocated, NULL-terminated string representing the current user
///
/// # Examples
///
/// ```c
/// printf("Hello, world\n");
/// ```
#[no_mangle]
#[allow(dead_code)]
#[link(name = "sqlite3", kind = "static")]
#[link(name = "sodium", kind = "static")]
pub extern "C" fn sdsync_initialize(db_path: *const std::os::raw::c_char, unique_client_id: *const std::os::raw::c_char) -> *mut CContext {
    let dbs: &CStr = unsafe {
        assert!(!db_path.is_null());
        CStr::from_ptr(db_path)
    };

    let db_directory: String = str::from_utf8(dbs.to_bytes()).unwrap().to_owned();

    let uids: &CStr = unsafe {
        assert!(!unique_client_id.is_null());
        CStr::from_ptr(unique_client_id)
    };

    let uid: String = str::from_utf8(uids.to_bytes()).unwrap().to_owned();

    let (db_path, storage_path, unique_client_path, unique_client_id) = initialize(db_directory, uid);

    let context = Context {
        db_path: db_path,
        storage_path: storage_path,
        unique_client_path: unique_client_path,
        unique_client_id: unique_client_id,
        ssh_username: None,
        ssh_password: None,
        safedrive_sftp_client_ip: None,
        safedrive_sftp_client_port: None,
        main_key: None,
        hmac_key: None
    };
    let ccontext = CContext(context);
    Box::into_raw(Box::new(ccontext))
}


/// Load keys, must be called before any other function that takes a context parameter
///
/// Will assert non-null on each parameter
///
/// Keys should be obtained from calls to sdsync_generate_symmetric_key() and persisted on disk
///
/// Parameters:
///
///     context: an opaque pointer obtained from calling sdsync_initialize()
///
///     main: a stack-allocated pointer to a 32byte main encryption key
///
///     hmac: a stack-allocated pointer to a 32byte hmac key
///
///
/// Return:
///
///     0: success
///
///     1+: failure
///
///
/// # Examples
///
/// ```c
/// uint8_t * main = malloc(32);
/// uint8_t * hmac = malloc(32)
/// sdsync_generate_symmetric_key(&main);
/// sdsync_generate_symmetric_key(&hmac);
/// sdsync_load_keys(&context, main_key, hmac_key);
/// free(main);
/// free(hmac);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_load_keys(context: *mut CContext, main: *const u8, hmac: *const u8) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!context.is_null()); &mut * context };

    let keysize = sodiumoxide::crypto::secretbox::KEYBYTES;

    let main_key = unsafe {
        assert!(!main.is_null());
        let mut array = [0u8; 32];
        let s = slice::from_raw_parts(main, keysize as usize);
        for (&x, p) in s.iter().zip(array.iter_mut()) {
            *p = x;
        };
        array
    };

    let hmac_key = unsafe {
        assert!(!hmac.is_null());
        let mut array = [0u8; 32];
        let s = slice::from_raw_parts(hmac, keysize as usize);
        for (&x, p) in s.iter().zip(array.iter_mut()) {
            *p = x;
        };
        array
    };

    c.0.set_keys(main_key, hmac_key);
    0
}


/// Load SSH credentials, must be called before any other function that creates or enumerates backups/chunks
///
/// Will assert non-null on each parameter
///
///
/// Parameters:
///
///     context: an opaque pointer obtained from calling sdsync_initialize()
///
///     username: a stack-allocated pointer to a username for sftp-client.safedrive.io
///
///     password: a stack-allocated pointer to a password for sftp-client.safedrive.io
///
///     ip_address: a stack-allocated pointer to ip
///
///     hmac: a stack-allocated pointer to a 32byte hmac key
///
///
/// Return:
///
///     0: success
///
///     1+: failure
///
///
/// # Examples
///
/// ```c
/// uint8_t * main = malloc(32);
/// uint8_t * hmac = malloc(32)
/// sdsync_generate_symmetric_key(&main);
/// sdsync_generate_symmetric_key(&hmac);
/// sdsync_load_keys(&context, main_key, hmac_key);
/// free(main);
/// free(hmac);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_load_credentials(context: *mut CContext,
                                      username: *const std::os::raw::c_char,
                                      password:  *const std::os::raw::c_char,
                                      ip_address:  *const std::os::raw::c_char,
                                      port: *const std::os::raw::c_char) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!context.is_null()); assert!(!username.is_null()); assert!(!password.is_null()); assert!(!ip_address.is_null()); assert!(!port.is_null()); &mut * context };

    let c_username: &CStr = unsafe { CStr::from_ptr(username) };
    let un: String = str::from_utf8(c_username.to_bytes()).unwrap().to_owned();

    let c_password: &CStr = unsafe { CStr::from_ptr(password) };
    let pa: String = str::from_utf8(c_password.to_bytes()).unwrap().to_owned();

    let c_ip: &CStr = unsafe { CStr::from_ptr(ip_address) };
    let ip: String = str::from_utf8(c_ip.to_bytes()).unwrap().to_owned();

    let c_port: &CStr = unsafe { CStr::from_ptr(port) };
    let p: String = str::from_utf8(c_port.to_bytes()).unwrap().to_owned();
    let po: u16 = u16::from_str(&p).unwrap();

    c.0.set_ssh_credentials(un, pa, ip, po);
    0
}

/// Generate a single key
///
/// Keys generated should be persisted on-disk for future use
///
/// Parameters:
///
///     buffer: an allocated pointer with exactly (32 * sizeof(uint8_t)) storage
///
///
/// Return:
///
///     0: success
///
///     1+: failure
///
///
/// # Examples
///
/// ```c
/// uint8_t * key = malloc((32 * sizeof(uint8_t)));
/// sdsync_generate_symmetric_key(&key);
/// // do something with the returned buffer here
/// free(key);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_generate_symmetric_key(mut buffer: *mut *mut u8) -> std::os::raw::c_int {
    // generate some randomness using libsodium
    // we need exactly 32 bytes
    let size = sodiumoxide::crypto::secretbox::KEYBYTES;

    let sample = sodiumoxide::randombytes::randombytes(size);
    unsafe {
        *buffer = sample.as_ptr() as *mut u8;
        mem::forget(sample);
    }
    0
}


/// Add a sync folder to the local database
///
/// Will return a failure code if for any reason the folder cannot be added to the database
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
///
/// Parameters:
///
///     context: an opaque pointer obtained from calling sdsync_initialize()
///
///     name: a stack-allocated, NULL terminated string representing the folder name
///
///     path: a stack-allocated, NULL terminated string representing the folder path in RFC3986 format
///
/// Return:
///
///     0: success
///
///     1+: failure
///
///
/// # Examples
///
/// ```c
/// int success = sdsync_add_sync_folder(&context, "Documents", "/Users/name/Documents");
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_add_sync_folder(context: *mut CContext,
                                     name: *const std::os::raw::c_char,
                                     path: *const std::os::raw::c_char) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!context.is_null()); &mut * context };

    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = str::from_utf8(c_name.to_bytes()).unwrap().to_owned();

    let c_path: &CStr = unsafe { CStr::from_ptr(path) };
    let p: String = str::from_utf8(c_path.to_bytes()).unwrap().to_owned();
    let db = Path::new(&c.0.db_path).to_owned();
    let result = add_sync_folder(db, &n, &p);
    match result {
        true => return 0,
        false => return 1,
    }
}


/// Get a list of all sync folders from the local database
///
/// The caller does not own the memory allocated by the library, it must be returned and freed by
/// the library after use. As a result, any data that the caller wishes to retain must be copied out
/// of the buffer before it is freed.
///
///
/// Parameters:
///
///     context: an opaque pointer obtained from calling sdsync_initialize()
///
///     folders: an uninitialized pointer that will be allocated and initialized when the function returns
///
/// Return:
///
///     -1: failure
///
///      0: no folders
///
///     1+: number of registered folders found, and number of CFolder structs allocated in folders
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
/// # Examples
///
/// ```c
/// sdsync_folders_t * folders = NULL;
/// int length = sdsync_get_sync_folders(&context, &folders);
/// // do something with folders here
/// sdsync_free_sync_folders(&folders, length);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_get_sync_folders(context: *mut CContext, mut folders: *mut *mut CFolder) -> u64 {
    let c = unsafe{ assert!(!context.is_null()); &mut * context };
    let db = &(c.0.db_path);

    let mut f = sync_folders(db).into_iter().map(|folder| {
        let s_name = Box::new(CString::new(folder.name.as_str()).unwrap());
        let s_path = Box::new(CString::new(folder.path.as_str()).unwrap());
        forget(&s_name);
        forget(&s_path);
        CFolder {
            id: folder.id,
            name: s_name.into_raw(),
            path: s_path.into_raw(),
        }
    }).collect::<Vec<CFolder>>();;

    f.shrink_to_fit();
    let len = f.len() as u64;
    unsafe {
        // `folders` must be passed back to sdsync_free_folders() after use
        // as the C code does not own the data, the caller must copy it before returning it to Rust
        // stack version
        *folders = f.as_ptr() as *mut CFolder;
        mem::forget(f);
        // heap version
        //*folders = libc::malloc(f.len() * std::mem::size_of::<CFolder>()) as *mut models::CFolder;
        //ptr::copy_nonoverlapping(f.as_ptr(), *folders, f.len());
    }
    len
}



/// Get a list of all sync sessions in the local database
///
/// The caller does not own the memory allocated by the library, it must be returned and freed by
/// the library after use. As a result, any data that the caller wishes to retain must be copied out
/// of the buffer before it is freed.
///
///
/// Parameters:
///
///     context: an opaque pointer obtained from calling sdsync_initialize()
///
///     folder_id: a stack-allocated integer for an existing registered folder name
///
///     sessions: an uninitialized pointer that will be allocated and initialized when the function returns
///
///
/// Return:
///
///     -1: failure
///
///      0: no sessions
///
///     1+: number of sessions found, and number of CSyncSession structs allocated in sessions
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
/// # Examples
///
/// ```c
/// sdsync_session_t * sessions = NULL;
/// int length = sdsync_get_sync_sessions(&context, 5, &sessions);
/// // do something with sessions here
/// sdsync_free_sync_sessions(&sessions, length);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_get_sync_sessions(context: *mut CContext, folder_id: std::os::raw::c_int, mut sessions: *mut *mut CSyncSession) -> u64 {
    let c = unsafe{ assert!(!context.is_null()); &mut * context };
    let db = &(c.0.db_path);
    let id: i32 = folder_id;



    let mut s = sync_sessions(db, id).into_iter().map(|ses| {
        let s_id = ses.id;
        let folder_id = ses.folder_id;
        let s_filename = Box::new(CString::new(ses.filename.as_str()).unwrap());

        let s_date = ses.date.sec as f64 + (ses.date.nsec as f64 / 1000.0 / 1000.0 / 1000.0 );

        forget(&folder_id);
        forget(&s_filename);
        forget(&s_id);
        CSyncSession {
            id: s_id,
            folder_id: folder_id,
            size: 0,
            filename: s_filename.into_raw(),
            date: s_date
        }
    }).collect::<Vec<CSyncSession>>();



    s.shrink_to_fit();
    let len = s.len() as u64;
    unsafe {
        // `sessions` must be passed back to sdsync_free_sessions() after use
        // as the C code does not own the data, the caller must copy it before returning it to Rust

        // stack version
        *sessions = s.as_ptr() as *mut CSyncSession;
        mem::forget(s);

        // heap version
        //*sessions = libc::malloc(s.len() * std::mem::size_of::<CSyncSession>()) as *mut models::CSyncSession;
        //ptr::copy_nonoverlapping(s.as_ptr(), *sessions, s.len());
    }
    len
}





/// Perform garbage collection of the local block cache and registry
///
/// Parameters:
///
///     context:  an opaque pointer obtained from calling sdsync_initialize()
///
/// Return:
///
///     0: success
///
///     1+: failure
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
/// # Examples
///
/// ```c
/// int result = sdsync_gc(&context);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_gc(context: *mut CContext) -> std::os::raw::c_int {
    let _ = unsafe{ assert!(!context.is_null()); &mut * context };
    0
}


/// Create a snapshot archive for the folder ID
///
///
/// Parameters:
///
///     context: an opaque pointer obtained from calling sdsync_initialize()
///
///     folder_id: a stack-allocated integer for an existing registered folder name
///
/// Return:
///
///     0: success
///
///     1+: failure
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
/// # Examples
///
/// ```c
/// int success = sdsync_create_archive(&context, "Documents");
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_create_archive(context: *mut CContext,
                                    folder_id: std::os::raw::c_int) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!context.is_null()); &mut * context };
    let main_key_array = (*c).0.get_main_key();
    let hmac_key_array = (*c).0.get_hmac_key();
    let ssh_username = (*c).0.get_ssh_username();
    let ssh_password = (*c).0.get_ssh_password();
    let safedrive_sftp_client_ip = (*c).0.get_safedrive_sftp_client_ip();
    let safedrive_sftp_client_port = (*c).0.get_safedrive_sftp_client_port();
    let unique_client_id = &c.0.unique_client_id;
    let db = Path::new(&c.0.db_path).to_owned();
    //let uid = &c.unique_client_id;
    let id: i32 = folder_id;

    match create_archive(main_key_array, hmac_key_array, ssh_username, ssh_password, safedrive_sftp_client_ip, safedrive_sftp_client_port, unique_client_id, db, id) {
        false => return 1,
        true => return 0
    }
}

/// Restore a snapshot archive for the named folder
///
/// Parameters:
///
///     context: an opaque pointer obtained from calling sdsync_initialize()
///
///     name: a stack-allocated, NULL-terminated string representing the registered folder name
///
///     destination: a stack-allocated, NULL-terminated string representing the path the archive will be unpacked to
///
/// Return:
///
///     0: success
///
///     1+: failure
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
/// # Examples
///
/// ```c
/// int success = sdsync_restore_archive(&context, "Documents");
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_restore_archive(context: *mut CContext,
                                     name: *const std::os::raw::c_char,
                                     destination: *const std::os::raw::c_char) -> std::os::raw::c_int {
    let _ = unsafe{ assert!(!context.is_null()); &mut * context };

    //let size = sodiumoxide::crypto::secretbox::KEYBYTES;

    //let main_key = (*c).main_key;

    //let hmac_key = (*c).hmac_key;

    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = str::from_utf8(c_name.to_bytes()).unwrap().to_owned();

    let c_destination: &CStr = unsafe { CStr::from_ptr(destination) };
    let d: String = str::from_utf8(c_destination.to_bytes()).unwrap().to_owned();

    println!("Rust<sdsync_restore_archive>: unpacking archive to {} for: {}", d, n);
    0
}












/// Free a pointer to a list of sync folders
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     folders: a pointer obtained from calling sdsync_get_sync_folders()
///
///     length: number of CFolder structs that were allocated in the pointer
///
///
/// # Examples
///
/// ```c
///sdsync_free_folders(&folders, length);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_free_folders(folders: *mut *mut CFolder, length: u64) {
    assert!(!folders.is_null());
    let l = length as usize;
    let _: Vec<CFolder> = unsafe { Vec::from_raw_parts(*folders, l, l) };
}

/// Free a pointer to a list of sync sessions
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     sessions: a pointer obtained from calling sdsync_get_sync_sessions()
///
///     length: number of CSyncSession structs that were allocated in the pointer
///
///
/// # Examples
///
/// ```c
///sdsync_free_sessions(&sessions, length);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_free_sessions(sessions: *mut *mut CSyncSession, length: u64) {
    assert!(!sessions.is_null());
    let l = length as usize;
    let _: Vec<CSyncSession> = unsafe { Vec::from_raw_parts(*sessions, l, l) };
}

/// Free a pointer to a key
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     key: a pointer obtained from calling sdsync_generate_symmetric_key()
///
///
/// # Examples
///
/// ```c
///sdsync_free_symmetric_key(&key);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_free_symmetric_key(key: *mut *mut u8) {
    assert!(!key.is_null());
    let length = 32usize;
    let _: Vec<u8> = unsafe { Vec::from_raw_parts(*key, length, length) };
}

/// Free an opaque pointer to an sdsync_context_t
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     context: a pointer obtained from calling sdsync_initialize()
///
/// # Examples
///
/// ```c
///sdsync_free_context(&context);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_free_context(context: *mut *mut CContext) {
    assert!(!context.is_null());

    let _: Box<CContext> = unsafe { Box::from_raw((*context)) };
}

/// Free a pointer to a string
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
///
/// Parameters:
///
///     string: a pointer obtained from calling a function that returns CFolders
///
/// # Examples
///
/// ```c
///sdsync_free_string(&string);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_free_string(string: *mut *mut std::os::raw::c_char) {
    assert!(!string.is_null());
    let _: CString = unsafe { CString::from_raw(*string) };
}

