use std;
use std::ffi::{CStr, CString};
use std::str;
use std::path::{Path, PathBuf};
use std::mem;

use std::u64;

// external imports

extern crate libc;

//internal imports

use ::state::State;

use ::core::initialize;

use ::core::add_sync_folder;
use ::core::remove_sync_folder;
use ::core::get_sync_folder;
use ::core::get_sync_folders;

use ::core::get_sync_sessions;
use ::core::create_archive;
use ::core::load_keys;
use ::core::login;

use ::models::{Configuration, RegisteredFolder, SyncSession};

use ::util::unique_client_hash;

// exports



#[derive(Debug)]
#[repr(C)]
pub struct SDDKState(State);

#[derive(Debug)]
#[repr(C)]
pub struct SDDKFolder {
    pub id: u32,
    pub name: *const std::os::raw::c_char,
    pub path: *const std::os::raw::c_char,
}

impl From<RegisteredFolder> for SDDKFolder {
    fn from(folder: RegisteredFolder) -> SDDKFolder {
        SDDKFolder {
            id: folder.id,
            name: CString::new(folder.folderName.as_str()).unwrap().into_raw(),
            path: CString::new(folder.folderPath.as_str()).unwrap().into_raw(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKSyncSession {
    pub name: *const std::os::raw::c_char,
    pub size: u64,
    pub date: u64,
    pub folder_id: u32,
}

impl From<SyncSession> for SDDKSyncSession {
    fn from(session: SyncSession) -> SDDKSyncSession {
        SDDKSyncSession {
            folder_id: session.folder_id,
            size: session.size,
            name: CString::new(session.name.as_str()).unwrap().into_raw(),
            date: session.time
        }
    }
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

    let storage_directory: String = match lstorage.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

    let uids: &CStr = unsafe {
        assert!(!unique_client_id.is_null());
        CStr::from_ptr(unique_client_id)
    };

    let uid: String = match uids.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

    let (storage_path, client_id) = initialize(storage_directory, uid);

    let state = State {
        storage_path: storage_path,
        unique_client_id: client_id,
        api_token: None,
        username: None,
        password: None,
        main_key: None,
        hmac_key: None,
        tweak_key: None,
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
    let un: String =  match c_username.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };


    let c_password: &CStr = unsafe { CStr::from_ptr(password) };
    let pa: String =  match c_password.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };


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
///     context: an opaque pointer the caller can pass in and get access to in the callback
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
pub extern "C" fn sddk_load_keys(context: *mut std::os::raw::c_void, state: *mut SDDKState, recovery_phrase: *const std::os::raw::c_char, store_recovery_key: extern fn(context: *mut std::os::raw::c_void, new_phrase: *const std::os::raw::c_char)) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!state.is_null()); &mut * state };

    let phrase: Option<String> = unsafe {
        if !state.is_null() {
            let c_recovery: &CStr = CStr::from_ptr(recovery_phrase);
            match c_recovery.to_str() {
                Ok(s) => Some(s.to_owned()),
                Err(e) => { panic!("string is not valid UTF-8: {}", e) },
            }
        }
        else {
            None
        }
    };

    let keyset = match load_keys(c.0.get_api_token(), phrase, &|new_phrase| {
        // call back to C to store phrase
        let c_new_phrase = CString::new(new_phrase).unwrap();
        store_recovery_key(context, c_new_phrase.into_raw());
    }) {
        Ok(ks) => ks,
        Err(_) => { return 1 }
    };

    c.0.set_keys(keyset.main, keyset.hmac, keyset.tweak);
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
    let e: String =  match c_email.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

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
    let n: String = match c_name.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };


    let c_path: &CStr = unsafe { CStr::from_ptr(path) };
    let p: String = match c_path.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };


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

    let id: u32 = folder_id as u32;


    match remove_sync_folder(c.0.get_api_token(), id) {
        Ok(_) => return 0,
        Err(_) => return 1,
    }
}

/// Get a sync folder from the SafeDrive server
///
/// The caller does not own the memory pointed to by `folder` after this function returns, it must
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
///     folder_id: a stack-allocated, unsigned 32-bit integer representing a registered folder ID
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
/// SDDKFolder * folder = NULL;
/// int res = sddk_get_sync_folder(&state, &folder);
/// // do something with folder here
/// sddk_free_sync_folder(&folder);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_sync_folder(state: *mut SDDKState, folder_id: std::os::raw::c_uint, mut folder: *mut *mut SDDKFolder) -> i8 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };
    let id: u32 = folder_id as u32;

    let nf = match get_sync_folder(c.0.get_api_token(), id) {
        Ok(folder) => folder,
        Err(e) => { error!("failed to get sync folder: {}", e); return -1 },
    };

    let f = SDDKFolder::from(nf);

    let b = Box::new(f);
    let ptr = Box::into_raw(b);

    unsafe {
        *folder = ptr;
    }

    0
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
/// SDDKFolder * folders = NULL;
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
        SDDKFolder::from(folder)
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
/// SDDKSession * sessions = NULL;
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

    let s = result.into_iter().map(|session| {
        SDDKSyncSession::from(session)
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
///     context: an opaque pointer the caller can pass in and get access to in the progress callback
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     name: a stack-allocated, NULL-terminated UUIDv4 string representing the name of the sync session
///
///     folder_id: a stack-allocated, unsigned 32-bit integer representing a registered folder ID
///
///     progress: a C function pointer that will be called periodically to report progress
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
/// int success = sddk_create_archive(NULL, &state, "02c0dc9c-6217-407b-a3ef-0d7ac5f288b1", 7, &fp);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_create_archive(context: *mut std::os::raw::c_void,
                                      state: *mut SDDKState,
                                      name: *const std::os::raw::c_char,
                                      folder_id: std::os::raw::c_uint,
                                      progress: extern fn(context: *mut std::os::raw::c_void, total: std::os::raw::c_uint, current: std::os::raw::c_uint, percent: std::os::raw::c_double, tick: std::os::raw::c_uint)) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };
    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = match c_name.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };


    let main_key = (*c).0.get_main_key();
    let hmac_key = (*c).0.get_hmac_key();
    let tweak_key = (*c).0.get_tweak_key();

    let id: u32 = folder_id as u32;

    match create_archive(c.0.get_api_token(),
                         &n,
                         main_key,
                         hmac_key,
                         tweak_key,
                         id,
                         &mut |total, current, progress_percent, tick| {
                             let c_total: std::os::raw::c_uint = total;
                             let c_current: std::os::raw::c_uint = current;
                             let c_percent: std::os::raw::c_double = progress_percent;
                             let c_tick: std::os::raw::c_uint =  if tick { 1 } else { 0 };
                             progress(context, c_total, c_current, c_percent, c_tick);
        }) {
        Ok(_) => return 0,
        Err(_) => return 1
    }
}

/// Restore a snapshot archive for the named folder
///
/// Parameters:
///
///     context: an opaque pointer the caller can pass in and get access to in the progress callback
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     name: a stack-allocated, NULL-terminated UTF-8 string representing the UUIDv4 name of the sync session
///
///     folder_id: a stack-allocated, unsigned 32-bit integer representing a registered folder ID
///
///     destination: a stack-allocated, NULL-terminated UTF-8 string representing the path the sync session will be restored to
///
///     progress: a C function pointer that will be called periodically to report progress
///
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
/// int success = sddk_restore_archive(NULL, &state, "02c0dc9c-6217-407b-a3ef-0d7ac5f288b1", 7, "/path/to/destination", &fp);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_restore_archive(context: *mut std::os::raw::c_void,
                                       state: *mut SDDKState,
                                       name: *const std::os::raw::c_char,
                                       folder_id: std::os::raw::c_uint,
                                       destination: *const std::os::raw::c_char,
                                       progress: extern fn(context: *mut std::os::raw::c_void, total: std::os::raw::c_uint, current: std::os::raw::c_uint, percent: std::os::raw::c_double, tick: std::os::raw::c_uint)) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };
    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = match c_name.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };


    let c_destination: &CStr = unsafe { CStr::from_ptr(destination) };
    let d: String =  match c_destination.to_str() {
        Ok(p) => p.to_owned(),
        Err(e) => { panic!("path is not valid UTF-8: {}", e) },
    };

    let main_key = (*c).0.get_main_key();
    let hmac_key = (*c).0.get_hmac_key();
    let id: u32 = folder_id as u32;


    debug!("unpacking archive {} to {}", n, d);
    0
}










/// Free a pointer to a sync folder
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     folder: a pointer obtained from calling sddk_get_sync_folder()
///
///
///
/// # Examples
///
/// ```c
///sddk_free_folder(&folder);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_free_folder(folder: *mut *mut SDDKFolder) {
    assert!(!folder.is_null());
    let folder: Box<SDDKFolder> = unsafe { Box::from_raw(*folder) };
    let _ = unsafe { CString::from_raw(folder.name as *mut i8) };
    let _ = unsafe { CString::from_raw(folder.path as *mut i8) };
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

