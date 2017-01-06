use std;
use std::ffi::{CStr, CString};
use std::str;
use std::path::{Path};
use std::mem;
use std::mem::forget;

use std::u64;
use std::str::FromStr;

// external imports

extern crate libc;

//internal imports

use ::context::Context;

use ::core::initialize;
use ::core::add_sync_folder;
use ::core::sync_folders;
use ::core::sync_sessions;
use ::core::create_archive;
use ::core::load_keys;
use ::core::login;

use ::util::unique_client_hash;

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
pub extern "C" fn sdsync_initialize(local_storage_path: *const std::os::raw::c_char, unique_client_id: *const std::os::raw::c_char) -> *mut CContext {
    let lstorage: &CStr = unsafe {
        assert!(!local_storage_path.is_null());
        CStr::from_ptr(local_storage_path)
    };

    let storage_directory: String = str::from_utf8(lstorage.to_bytes()).unwrap().to_owned();

    let uids: &CStr = unsafe {
        assert!(!unique_client_id.is_null());
        CStr::from_ptr(unique_client_id)
    };

    let uid: String = str::from_utf8(uids.to_bytes()).unwrap().to_owned();


    let (storage_path, client_id) = initialize(storage_directory, uid);

    let context = Context {
        storage_path: storage_path,
        unique_client_id: client_id,
        api_token: None,
        username: None,
        password: None,
        main_key: None,
        hmac_key: None
    };
    let ccontext = CContext(context);
    Box::into_raw(Box::new(ccontext))
}

/// Login to SafeDrive, must be called before any other function that interacts with the SFTP server
///
/// Will assert non-null on each parameter
///
///
/// Parameters:
///
///     context: an opaque pointer obtained from calling sdsync_initialize()
///
///     account_username: a stack-allocated pointer to a username for a SafeDrive account
///
///     account_password: a stack-allocated pointer to a password for a SafeDrive account
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
/// if (0 != sdsync_login(&context, "user@safedrive.io", "password")) {
///     printf("Login failed");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_login(context: *mut CContext,
                               username: *const std::os::raw::c_char,
                               password:  *const std::os::raw::c_char) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!context.is_null()); assert!(!username.is_null()); assert!(!password.is_null()); &mut * context };

    let c_username: &CStr = unsafe { CStr::from_ptr(username) };
    let un: String = str::from_utf8(c_username.to_bytes()).unwrap().to_owned();

    let c_password: &CStr = unsafe { CStr::from_ptr(password) };
    let pa: String = str::from_utf8(c_password.to_bytes()).unwrap().to_owned();

    match login(&un, &pa) {
        Ok((token, account_status, unique_client_id)) => {
            c.0.set_account(un.to_owned(), pa.to_owned());
            c.0.set_api_token(token);
            0
        },
        Err(_) => {
            1
        }
    }
}








/// Load keys, must be called before any other function that operates on archives
///
/// Will assert non-null on the context parameter only, the others will cause a crash if null
///
///
/// Parameters:
///
///     context: an opaque pointer obtained from calling sdsync_initialize()
///
///     recovery_phrase: a stack-allocated pointer to a recovery phrase obtained by previous calls.
///                      can be null if no phrase is available
///
///     store_recovery_key: a C function pointer that will be called when the app should store a
///                         recovery phrase
///
///     store_recovery_key(new_phrase): a pointer to a recovery phrase that should be stored by the app.
///                                     note that the pointer becomes invalid after the callback function
///                                     returns
///
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
/// ```
/// void store_recovery_key_cb(char const* new_phrase) {
///     // do something with new_phrase, pointer becomes invalid after return
/// }
/// sdsync_load_keys(&context, "genius quality lunch cost cream question remain narrow barely circle weapon ask", &store_recovery_key_cb);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_load_keys(context: *mut CContext, recovery_phrase: *const std::os::raw::c_char, store_recovery_key: extern fn(new_phrase: *const std::os::raw::c_char)) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!context.is_null()); &mut * context };

    let phrase: Option<String> = unsafe {
        if !context.is_null() {
            let c_recovery: &CStr = CStr::from_ptr(recovery_phrase);
            match str::from_utf8(c_recovery.to_bytes()) {
                Ok(s) => Some(s.to_owned()),
                Err(_) => None
            }
        }
        else {
            None
        }
    };

    let (_, main_key, hmac_key) = match load_keys(c.0.get_api_token(), phrase, &|new_phrase| {
        // call back to C to store phrase
        let c_new_phrase = CString::new(new_phrase).unwrap();
        store_recovery_key(c_new_phrase.into_raw());
    }) {
        Ok((master_key, main_key, hmac_key)) => (master_key, main_key, hmac_key),
        Err(_) => { return 1 }
    };

    c.0.set_keys(main_key, hmac_key);
    0
}

/// Generate a unique client ID using the users email address and store it in the `unique_client_id` parameter
///
/// Will return a failure code if for any reason the hash cannot be generated properly
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
///
/// Parameters:
///
///
///     email: a stack-allocated, NULL terminated string representing the user email address
///
///     unique_client_id: an allocated pointer with exactly 64bytes + 1byte NULL terminator storage,
///                       will be initialized by the library and must be freed by the caller
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
/// char * unique_client_id = malloc((64 * sizeof(char)) + 1);
/// int success = sdsync_get_unique_client_id("user@example.com");
/// free(unique_client_id);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sdsync_get_unique_client_id(email: *const std::os::raw::c_char,
                                              mut unique_client_id: *mut *mut std::os::raw::c_char) -> std::os::raw::c_int {

    let c_email: &CStr = unsafe { CStr::from_ptr(email) };
    let e: String = str::from_utf8(c_email.to_bytes()).unwrap().to_owned();

    match unique_client_hash(&e) {
        Ok(hash) => {
            let c_hash = Box::new(CString::new(hash).expect("Failed to get unique client id hash"));
            mem::forget(&c_hash);

            unsafe {
                *unique_client_id = c_hash.into_raw();
            }
            return 0
        },
        Err(_) => return 1,
    }
}

/// Add a sync folder
///
/// Will return a failure code if for any reason the folder cannot be added
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

    match add_sync_folder(&n, &p) {
        Ok(_) => return 0,
        Err(_) => return 1,
    }
}


/// Get a list of all sync folders from the SafeDive server
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
///     0+: number of registered folders found, and number of CFolder structs allocated in folders
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

    let result = match sync_folders() {
        Ok(folders) => folders,
        Err(e) => panic!("Rust<sdsync_get_sync_folders> failed to get list of sync folders: {}", e),
    };

    let mut f = result.into_iter().map(|folder| {
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
        // stack version, not safe at all
        *folders = f.as_ptr() as *mut CFolder;
        mem::forget(f);
        // malloc version
        //*folders = libc::malloc(f.len() * std::mem::size_of::<CFolder>()) as *mut models::CFolder;
        //ptr::copy_nonoverlapping(f.as_ptr(), *folders, f.len());

        // box version
        //let bf = Box::new(f);
        //mem::forget(bf);
        //*folders = bf as *mut CFolder;

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
    let id: i32 = folder_id;


    let result = match sync_sessions(id) {
        Ok(ses) => ses,
        Err(e) => panic!("Rust<sdsync_get_sync_sessions> failed to get list of sync sessions: {}", e),
    };

    let mut s = result.into_iter().map(|ses| {
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

        // stack version, not safe at all
        *sessions = s.as_ptr() as *mut CSyncSession;
        mem::forget(s);

        // malloc version
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
                                        name: *const std::os::raw::c_char,
                                        folder_path: *const std::os::raw::c_char,
                                        folder_id: std::os::raw::c_int,
                                        progress: extern fn(percent: std::os::raw::c_double)) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!context.is_null()); &mut * context };
    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = str::from_utf8(c_name.to_bytes()).unwrap().to_owned();

    let main_key = (*c).0.get_main_key();
    let hmac_key = (*c).0.get_hmac_key();
    let id: i32 = folder_id;

    let c_fpath: &CStr = unsafe { CStr::from_ptr(folder_path) };
    let fp: String = str::from_utf8(c_fpath.to_bytes()).unwrap().to_owned();

    let folder_path = Path::new(&fp).to_owned();

    match create_archive(c.0.get_api_token(),
                         &n,
                         main_key,
                         hmac_key,
                         id,
                         folder_path,
                         &mut |progress_percent| {
            let c_percent: std::os::raw::c_double = progress_percent;

            progress(c_percent);
        }) {
        Ok(_) => return 0,
        Err(_) => return 1
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

