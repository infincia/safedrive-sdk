use std;
use std::ffi::{CStr, CString};
use std::str;
use std::path::{Path};
use std::mem;

use std::u64;

// external imports

extern crate libc;

//internal imports

use ::state::State;

use ::core::initialize;

use ::core::add_sync_folder;
use ::core::remove_sync_folder;
use ::core::get_sync_folders;

use ::core::get_sync_sessions;
use ::core::create_archive;
use ::core::load_keys;
use ::core::login;

use ::models::Configuration;

use ::util::unique_client_hash;

// exports



#[derive(Debug)]
#[repr(C)]
pub struct SDDKState(State);

#[derive(Debug)]
#[repr(C)]
pub struct SDDKFolder {
    pub id: i64,
    pub name: *const std::os::raw::c_char,
    pub path: *const std::os::raw::c_char,
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKSyncSession {
    pub name: *const std::os::raw::c_char,
    pub size: u64,
    pub date: u64,
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
pub extern "C" fn sddk_initialize(local_storage_path: *const std::os::raw::c_char, unique_client_id: *const std::os::raw::c_char) -> *mut SDDKState {
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

    let state = State {
        storage_path: storage_path,
        unique_client_id: client_id,
        api_token: None,
        username: None,
        password: None,
        main_key: None,
        hmac_key: None,
        config: Configuration::Production
    };
    let cstate = SDDKState(state);
    Box::into_raw(Box::new(cstate))
}

/// Login to SafeDrive, must be called before any other function that interacts with the SFTP server
///
/// Will assert non-null on each parameter
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
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
/// if (0 != sddk_login(&state, "user@safedrive.io", "password")) {
///     printf("Login failed");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_login(state: *mut SDDKState,
                               username: *const std::os::raw::c_char,
                               password:  *const std::os::raw::c_char) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!state.is_null()); assert!(!username.is_null()); assert!(!password.is_null()); &mut * state };

    let c_username: &CStr = unsafe { CStr::from_ptr(username) };
    let un: String = str::from_utf8(c_username.to_bytes()).unwrap().to_owned();

    let c_password: &CStr = unsafe { CStr::from_ptr(password) };
    let pa: String = str::from_utf8(c_password.to_bytes()).unwrap().to_owned();

    match login(&un, &pa) {
        Ok((token, _, _)) => {
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
/// Will assert non-null on the state parameter only, the others will cause a crash if null
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
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
/// sddk_load_keys(&state, "genius quality lunch cost cream question remain narrow barely circle weapon ask", &store_recovery_key_cb);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_load_keys(state: *mut SDDKState, recovery_phrase: *const std::os::raw::c_char, store_recovery_key: extern fn(new_phrase: *const std::os::raw::c_char)) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!state.is_null()); &mut * state };

    let phrase: Option<String> = unsafe {
        if !state.is_null() {
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
/// int success = sddk_get_unique_client_id("user@example.com");
/// free(unique_client_id);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_unique_client_id(email: *const std::os::raw::c_char,
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
///     state: an opaque pointer obtained from calling sddk_initialize()
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
/// int success = sddk_add_sync_folder(&state, "Documents", "/Users/name/Documents");
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_add_sync_folder(state: *mut SDDKState,
                                         name: *const std::os::raw::c_char,
                                         path: *const std::os::raw::c_char) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = str::from_utf8(c_name.to_bytes()).unwrap().to_owned();

    let c_path: &CStr = unsafe { CStr::from_ptr(path) };
    let p: String = str::from_utf8(c_path.to_bytes()).unwrap().to_owned();

    match add_sync_folder(c.0.get_api_token(), &n, &p) {
        Ok(_) => return 0,
        Err(_) => return 1,
    }
}

/// Remove a sync folder
///
/// Will return a failure code if for any reason the folder cannot be removed
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     folder_id: a stack-allocated, unsigned 32-bit integer representing the registered folder ID
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
/// int success = sddk_remove_sync_folder(&state, 7);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_remove_sync_folder(state: *mut SDDKState,
                                          folder_id: std::os::raw::c_uint) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let id: i32 = folder_id as i32;


    match remove_sync_folder(c.0.get_api_token(), id) {
        Ok(_) => return 0,
        Err(_) => return 1,
    }
}


/// Get a list of all sync folders from the SafeDrive server
///
/// The caller does not own the memory pointed to by `folders` after this function returns, it must
/// be returned and freed by the library.
///
/// As a result, any data that the caller wishes to retain must be copied out of the buffer before
/// it is freed.
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     folders: an uninitialized pointer that will be allocated and initialized when the function returns
///
/// Return:
///
///     -1: failure
///
///     0+: number of registered folders found, and number of SDDKFolder structs allocated in folders
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
/// # Examples
///
/// ```c
/// sddk_folders_t * folders = NULL;
/// int length = sddk_get_sync_folders(&state, &folders);
/// // do something with folders here
/// sddk_free_sync_folders(&folders, length);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_sync_folders(state: *mut SDDKState, mut folders: *mut *mut SDDKFolder) -> i64 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let result = match get_sync_folders(c.0.get_api_token()) {
        Ok(folders) => folders,
        Err(e) => { error!("failed to get list of sync folders: {}", e); return -1 },
    };

    let f = result.into_iter().map(|folder| {
        SDDKFolder {
            id: folder.id as i64,
            name: CString::new(folder.folderName.as_str()).unwrap().into_raw(),
            path: CString::new(folder.folderPath.as_str()).unwrap().into_raw(),
        }
    }).collect::<Vec<SDDKFolder>>();;

    let mut b = f.into_boxed_slice();
    let ptr = b.as_mut_ptr();
    let len = b.len();
    std::mem::forget(b);

    unsafe {
        *folders = ptr;
    }
    
    len as i64
}



/// Get a list of all sync sessions from the SafeDrive server
///
/// The caller does not own the memory pointed to by `sessions` after this function returns, it must
/// be returned and freed by the library.
///
/// As a result, any data that the caller wishes to retain must be copied out of the buffer before
/// it is freed.
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     sessions: an uninitialized pointer that will be allocated and initialized when the function returns
///
///
/// Return:
///
///     -1: failure
///
///      0+: number of sessions found, and number of SDDKSyncSession structs allocated in sessions
///
/// Return codes will be expanded in the future to provide more specific information on the failure
///
/// # Examples
///
/// ```c
/// sddk_session_t * sessions = NULL;
/// int length = sddk_get_sync_sessions(&state, &sessions);
/// // do something with sessions here
/// sddk_free_sync_sessions(&sessions, length);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_sync_sessions(state: *mut SDDKState, mut sessions: *mut *mut SDDKSyncSession) -> i64 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let result = match get_sync_sessions(c.0.get_api_token()) {
        Ok(ses) => ses,
        Err(e) => { error!("failed to get list of sync sessions: {}", e); return -1 },
    };

    let s = result.into_iter().map(|ses| {
        SDDKSyncSession {
            folder_id: ses.folder_id,
            size: ses.size,
            name: CString::new(ses.name.as_str()).unwrap().into_raw(),
            date: ses.time
        }
    }).collect::<Vec<SDDKSyncSession>>();

    let mut b = s.into_boxed_slice();
    let ptr = b.as_mut_ptr();
    let len = b.len();
    std::mem::forget(b);

    unsafe {
        *sessions = ptr;
    }

    len as i64
}





/// Perform garbage collection of the local block cache and registry
///
/// Parameters:
///
///     state:  an opaque pointer obtained from calling sddk_initialize()
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
/// int result = sddk_gc(&state);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_gc(state: *mut SDDKState) -> std::os::raw::c_int {
    let _ = unsafe{ assert!(!state.is_null()); &mut * state };
    0
}


/// Create a snapshot archive for the folder ID
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     folder_id: a stack-allocated, unsigned 32-bit integer representing a registered folder ID
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
/// int success = sddk_create_archive(&state, "Documents");
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_create_archive(state: *mut SDDKState,
                                        name: *const std::os::raw::c_char,
                                        folder_path: *const std::os::raw::c_char,
                                        folder_id: std::os::raw::c_uint,
                                        progress: extern fn(total: std::os::raw::c_uint, current: std::os::raw::c_uint, percent: std::os::raw::c_double, tick: std::os::raw::c_uint)) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };
    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = str::from_utf8(c_name.to_bytes()).unwrap().to_owned();

    let main_key = (*c).0.get_main_key();
    let hmac_key = (*c).0.get_hmac_key();
    let id: i32 = folder_id as i32;

    let c_fpath: &CStr = unsafe { CStr::from_ptr(folder_path) };
    let fp: String = str::from_utf8(c_fpath.to_bytes()).unwrap().to_owned();

    let folder_path = Path::new(&fp).to_owned();

    match create_archive(c.0.get_api_token(),
                         &n,
                         main_key,
                         hmac_key,
                         id,
                         folder_path,
                         &mut |total, current, progress_percent, tick| {
                             let c_total: std::os::raw::c_uint = total;
                             let c_current: std::os::raw::c_uint = current;
                             let c_percent: std::os::raw::c_double = progress_percent;
                             let c_tick: std::os::raw::c_uint =  if tick { 1 } else { 0 };
                             progress(c_total, c_current, c_percent, c_tick);
        }) {
        Ok(_) => return 0,
        Err(_) => return 1
    }
}

/// Restore a snapshot archive for the named folder
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
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
/// int success = sddk_restore_archive(&state, "Documents");
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_restore_archive(state: *mut SDDKState,
                                         name: *const std::os::raw::c_char,
                                         destination: *const std::os::raw::c_char) -> std::os::raw::c_int {
    let _ = unsafe{ assert!(!state.is_null()); &mut * state };

    //let size = sodiumoxide::crypto::secretbox::KEYBYTES;

    //let main_key = (*c).main_key;

    //let hmac_key = (*c).hmac_key;

    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = str::from_utf8(c_name.to_bytes()).unwrap().to_owned();

    let c_destination: &CStr = unsafe { CStr::from_ptr(destination) };
    let d: String = str::from_utf8(c_destination.to_bytes()).unwrap().to_owned();

    debug!("unpacking archive to {} for: {}", d, n);
    0
}












/// Free a pointer to a list of sync folders
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     folders: a pointer obtained from calling sddk_get_sync_folders()
///
///     length: number of SDDKFolder structs that were allocated in the pointer
///
///
/// # Examples
///
/// ```c
///sddk_free_folders(&folders, length);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_free_folders(folders: *mut *mut SDDKFolder, length: u64) {
    assert!(!folders.is_null());
    let l = length as usize;

    let folders: Vec<SDDKFolder> = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(*folders, l)).into_vec() };
    for folder in folders {
        let _ = unsafe { CString::from_raw(folder.name as *mut i8) };
        let _ = unsafe { CString::from_raw(folder.path as *mut i8) };
    }
}

/// Free a pointer to a list of sync sessions
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     sessions: a pointer obtained from calling sddk_get_sync_sessions()
///
///     length: number of SDDKSyncSession structs that were allocated in the pointer
///
///
/// # Examples
///
/// ```c
///sddk_free_sync_sessions(&sessions, length);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_free_sync_sessions(sessions: *mut *mut SDDKSyncSession, length: u64) {
    assert!(!sessions.is_null());
    let l = length as usize;
    let sessions: Vec<SDDKSyncSession> = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(*sessions, l)).into_vec() };
    for session in sessions {
        let _ = unsafe { CString::from_raw(session.name as *mut i8) };
    }
}

/// Free an opaque pointer to an SDDKState
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     state: a pointer obtained from calling sddk_initialize()
///
/// # Examples
///
/// ```c
///sddk_free_state(&state);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_free_state(state: *mut *mut SDDKState) {
    assert!(!state.is_null());

    let _: Box<SDDKState> = unsafe { Box::from_raw((*state)) };
}

/// Free a pointer to a string
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
///
/// Parameters:
///
///     string: a pointer obtained from calling a function that returns SDDKFolders
///
/// # Examples
///
/// ```c
///sddk_free_string(&string);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_free_string(string: *mut *mut std::os::raw::c_char) {
    assert!(!string.is_null());
    let _: CString = unsafe { CString::from_raw(*string) };
}

