#![allow(unused_variables)]
#![allow(unused_mut)]

use std;
use std::ffi::{CStr, CString};
use std::str;
use std::path::{Path, PathBuf};

use std::u64;

// internal imports

use ::state::State;

use ::core::initialize;

use ::core::add_sync_folder;
use ::core::remove_sync_folder;
use ::core::get_sync_folder;
use ::core::get_sync_folders;

use ::core::get_sync_sessions;
use ::core::sync;
use ::core::restore;
use ::core::load_keys;
use ::core::login;

use ::models::{Configuration, RegisteredFolder};

use ::session::SyncSession;

use ::core::unique_client_hash;

use ::error::SDError;

// exports

#[derive(Debug)]
#[repr(C)]
pub struct SDDKState(State);

#[derive(Debug)]
#[repr(C)]
pub enum SDDKConfiguration {
    SDDKConfigurationProduction,
    SDDKConfigurationStaging,
}

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
            folder_id: session.folder_id.unwrap(),
            size: session.size.unwrap(),
            name: CString::new(session.name.as_str()).unwrap().into_raw(),
            date: session.time.unwrap(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum SDDKErrorType {
    Internal = 0x0001,
    RequestFailure = 0x0002,
    NetworkFailure = 0x0003,
    Conflict = 0x0004,
    BlockMissing = 0x0005,
    SessionMissing = 0x0006,
    RecoveryPhraseIncorrect = 0x0007,
    InsufficientFreeSpace = 0x0008,
    Authentication = 0x0009,
    UnicodeError = 0x000A,
    TokenExpired = 0x000B,
    CryptoError = 0x000C,
    IO = 0x000D,
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKError {
    pub message: *const std::os::raw::c_char,
    pub error_type: SDDKErrorType,
}

impl From<SDError> for SDDKError {
    fn from(e: SDError) -> Self {
        let error_type = match e {
            SDError::Internal(_) => {
                SDDKErrorType::Internal
            },
            SDError::IO(_) => {
                SDDKErrorType::IO
            },
            SDError::RequestFailure(_) => {
                SDDKErrorType::RequestFailure
            },
            SDError::NetworkFailure(_) => {
                SDDKErrorType::NetworkFailure
            },
            SDError::Conflict(_) => {
                SDDKErrorType::Conflict
            },
            SDError::BlockMissing => {
                SDDKErrorType::Internal
            },
            SDError::SessionMissing => {
                SDDKErrorType::Internal
            },
            SDError::RecoveryPhraseIncorrect => {
                SDDKErrorType::RecoveryPhraseIncorrect
            },
            SDError::InsufficientFreeSpace => {
                SDDKErrorType::InsufficientFreeSpace
            },
            SDError::Authentication => {
                SDDKErrorType::Authentication
            },
            SDError::UnicodeError => {
                SDDKErrorType::UnicodeError
            },
            SDError::TokenExpired => {
                SDDKErrorType::TokenExpired
            },
            SDError::CryptoError(_) => {
                SDDKErrorType::CryptoError
            },
        };
        SDDKError { error_type: error_type, message: CString::new(format!("{}", e)).unwrap().into_raw() }
    }
}

impl From<str::Utf8Error> for SDDKError {
    fn from(e: str::Utf8Error) -> Self {
        SDDKError { error_type: SDDKErrorType::UnicodeError, message: CString::new(format!("{}", e)).unwrap().into_raw() }
    }
}


impl From<String> for SDDKError {
    fn from(s: String) -> Self {
        SDDKError { error_type: SDDKErrorType::Internal, message: CString::new(format!("{}", s)).unwrap().into_raw() }
    }
}


/// Initialize the library, must be called before any other function.
///
/// If the application needs to switch users or the unique client ID changes, free the SDDKState and
/// call initialize again
///
/// Will assert non-null on `local_storage_path` and `unique_client_id`
///
/// Parameters:
///
///     local_storage_path: a stack-allocated, NULL-terminated string representing the location the app can store settings for a user
///
///     unique_client_id: a stack-allocated, NULL-terminated string representing the current unique_client_id
///
///     config: a stack-allocated enum variant of SDDKConfiguration which controls API endpoint and other things
///
///             use SDDKConfigurationStaging for the staging environment
///             use SDDKConfigurationProduction for the production environment
///
///     state: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was 0
///
///            must be freed by the caller using sddk_free_state()
///
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
///
///     -1: failure, `error` will be set with more information
///
///      0: success
///
/// # Examples
///
/// ```c
/// SDDKState *state = sddk_initialize("/home/user/.safedrive/", "12345", SDDKConfigurationProduction)
/// // do something with SDDKState here, store it somewhere, then free it later on
/// sddk_free_state(&state);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_initialize(local_storage_path: *const std::os::raw::c_char,
                                  unique_client_id: *const std::os::raw::c_char,
                                  config: SDDKConfiguration) -> *mut SDDKState {
    let lstorage: &CStr = unsafe {
        assert!(!local_storage_path.is_null());
        CStr::from_ptr(local_storage_path)
    };

    let storage_directory: String = match lstorage.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

    let storage_path = Path::new(&storage_directory);

    let uids: &CStr = unsafe {
        assert!(!unique_client_id.is_null());
        CStr::from_ptr(unique_client_id)
    };

    let uid: String = match uids.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

    let c = match config {
        SDDKConfiguration::SDDKConfigurationProduction => Configuration::Production,
        SDDKConfiguration::SDDKConfigurationStaging => Configuration::Staging,
    };

    initialize(storage_path, c);

    let sstate = State::new(storage_path.to_owned(), uid);
    let c_state = SDDKState(sstate);

    let b = Box::new(c_state);
    let ptr = Box::into_raw(b);

    ptr
}

/// Login to SafeDrive, must be called before any other function that interacts with the SFTP server
///
/// Will assert non-null on `state`, `username`, and `password`
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     username: a stack-allocated, NULL-terminated string representing a username for a SafeDrive account
///
///     password: a stack-allocated, NULL-terminated string representing a password for a SafeDrive account
///
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///      0: success
///
///
/// # Examples
///
/// ```c
/// SDDKState *state; //retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// if (0 != sddk_login(&state, "user@safedrive.io", "password", &error)) {
///     printf("Login failed");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Login successful");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_login(state: *mut SDDKState,
                             username: *const std::os::raw::c_char,
                             password:  *const std::os::raw::c_char,
                             mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
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

    match login(c.0.get_unique_client_id(), &un, &pa) {
        Ok((token, _, _)) => {
            c.0.set_account(un.to_owned(), pa.to_owned());
            c.0.set_api_token(token);
            0
        },
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        }
    }
}








/// Load keys, must be called before any other function that starts a sync or restore
///
/// Will assert non-null on the `state` parameter only
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
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///      0: success
///
///
/// # Examples
///
/// ```
/// void *context; // arbitrary, can be anything you want access to in the callback
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// void store_recovery_key_cb(char const* new_phrase) {
///     // do something with new_phrase, pointer becomes invalid after return
/// }
///
/// if (0 != sddk_load_keys(&context, &state, &error, "genius quality lunch cost cream question remain narrow barely circle weapon ask", &store_recovery_key_cb)) {
///     printf("Loading keys failed");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Loading keys successful");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_load_keys(context: *mut std::os::raw::c_void,
                                 state: *mut SDDKState,
                                 mut error: *mut *mut SDDKError,
                                 recovery_phrase: *const std::os::raw::c_char,
                                 store_recovery_key: extern fn(context: *mut std::os::raw::c_void,
                                                               new_phrase: *mut std::os::raw::c_char)) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!state.is_null()); &mut * state };

    let phrase: Option<String> = unsafe {
        if !recovery_phrase.is_null() {
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
        let mut c_new_phrase = CString::new(new_phrase).unwrap();
        store_recovery_key(context, c_new_phrase.into_raw());
    }) {
        Ok(ks) => ks,
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1
        }
    };

    c.0.set_keys(keyset.main, keyset.hmac, keyset.tweak);
    0
}

/// Generate a unique client ID using the users email address and store it in the `unique_client_id` parameter
///
/// Will return a failure code if for any reason the hash cannot be generated properly
///
/// The caller does not own the memory pointed to by `unique_client_id` after this function returns, it must
/// be returned and freed by the library.
///
/// As a result, any data that the caller wishes to retain must be copied out of the buffer before
/// it is freed.
///
///
/// Parameters:
///
///
///     email: a stack-allocated, NULL terminated string representing the user email address
///
///     unique_client_id: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was 0
///
///            must be freed by the caller using sddk_free_string()
///
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///      0: success
///
///
/// # Examples
///
/// ```c
/// char * unique_client_id;
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// if (0 != sddk_get_unique_client_id("user@example.com", &unique_client_id, &error)) {
///     printf("Failed to get unique client id");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Got unique client id");
///     // do something with unique_client_id here, then free it
///     sddk_free_string(&unique_client_id);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_unique_client_id(email: *const std::os::raw::c_char,
                                            mut unique_client_id: *mut *mut std::os::raw::c_char,
                                            mut error: *mut *mut SDDKError) -> std::os::raw::c_int {

    let c_email: &CStr = unsafe { CStr::from_ptr(email) };
    let e: String =  match c_email.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

    match unique_client_hash(&e) {
        Ok(hash) => {
            unsafe {
                *unique_client_id = CString::new(hash).expect("Failed to get unique client id hash").into_raw();
            }
            0
        },
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Add a sync folder
///
/// Will return a failure code if for any reason the folder cannot be added
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
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///      0: success
///
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// if (0 != sddk_add_sync_folder(&state, "Documents", "/Users/name/Documents", &error)) {
///     printf("Failed to add folder");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Added folder");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_add_sync_folder(state: *mut SDDKState,
                                       name: *const std::os::raw::c_char,
                                       path: *const std::os::raw::c_char,
                                       mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
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
        Ok(_) => 0,
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Remove a sync folder
///
/// Will return a failure code and set the error parameter if for any reason the folder cannot be removed
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     folder_id: a stack-allocated, unsigned 32-bit integer representing the registered folder ID
///
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///      0: success
///
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// if (0 != sddk_remove_sync_folder(&state, 7, &error)) {
///     printf("Failed to remove folder");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Removed folder");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_remove_sync_folder(state: *mut SDDKState,
                                          folder_id: std::os::raw::c_uint,
                                          mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let id: u32 = folder_id as u32;


    match remove_sync_folder(c.0.get_api_token(), id) {
        Ok(_) => 0,
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Get a sync folder from the SafeDrive server
///
/// The caller does not own the memory pointed to by `folder` or `error` after this function returns,
/// they must be returned and freed by the library.
///
/// As a result, any data that the caller wishes to retain must be copied out of the buffers before
/// they are freed.
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     folder_id: a stack-allocated, unsigned 32-bit integer representing a registered folder ID
///
///     folder: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was 0
///
///            must be freed by the caller using sddk_free_folder()
///
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///     0+: number of registered folders found, and number of SDDKFolder structs allocated in folders
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
/// SDDKFolder * folder = NULL;
///
/// if (0 != sddk_get_sync_folder(&state, 7, &folder, &error)) {
///     printf("Failed to get folder");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Got folder");
///     // do something with folder here, then free it
///     sddk_free_sync_folder(&folder);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_sync_folder(state: *mut SDDKState,
                                       folder_id: std::os::raw::c_uint,
                                       mut folder: *mut *mut SDDKFolder,
                                       mut error: *mut *mut SDDKError) -> i8 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };
    let id: u32 = folder_id as u32;

    let nf = match get_sync_folder(c.0.get_api_token(), id) {
        Ok(folder) => folder,
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1
        },
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
///     folders: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was 0
///
///            must be freed by the caller using sddk_free_folders()
///
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///     0+: number of registered folders found, and number of SDDKFolder structs allocated in folders
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
/// SDDKFolder * folders = NULL;
///
/// int length = sddk_get_folders(&state, &folders, &error);
///
/// if (0 != length) {
///     printf("Failed to get folders");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Got folders");
///     // do something with folders here, then free it
///     sddk_free_folders(&folders, length);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_sync_folders(state: *mut SDDKState,
                                        mut folders: *mut *mut SDDKFolder,
                                        mut error: *mut *mut SDDKError) -> i64 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let result = match get_sync_folders(c.0.get_api_token()) {
        Ok(folders) => folders,
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1
        },
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
///     sessions: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was 0
///
///            must be freed by the caller using sddk_free_sessions()
///
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///      -1: failure, `error` will be set with more information
///
///      0+: number of sessions found, and number of SDDKSyncSession structs allocated in sessions
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
/// SDDKSession * sessions = NULL;
///
/// int length = sddk_get_sync_sessions(&state, &sessions, &error);
///
/// if (0 != length) {
///     printf("Failed to get sessions");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Got sessions");
///     // do something with sessions here, then free it
///     sddk_free_sync_sessions(&sessions, length);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_sync_sessions(state: *mut SDDKState,
                                         mut sessions: *mut *mut SDDKSyncSession,
                                         mut error: *mut *mut SDDKError) -> i64 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let result = match get_sync_sessions(c.0.get_api_token()) {
        Ok(ses) => ses,
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1
        },
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
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///      0: success
///
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// if (0 != sddk_gc(&state)) {
///     printf("Failed to run GC");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("GC successful");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_gc(state: *mut SDDKState,
                          mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
    let _ = unsafe{ assert!(!state.is_null()); &mut * state };
    0
}


/// Start a sync for the folder ID
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
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///      0: success
///
/// # Examples
///
/// ```c
/// void *context; // arbitrary, can be anything you want access to in the callback
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// if (0 != sddk_sync(&context, &state, &error, "02c0dc9c-6217-407b-a3ef-0d7ac5f288b1", 7, &fp)) {
///     printf("Failed to sync");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Sync successful");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_sync(context: *mut std::os::raw::c_void,
                            state: *mut SDDKState,
                            mut error: *mut *mut SDDKError,
                            name: *const std::os::raw::c_char,
                            folder_id: std::os::raw::c_uint,
                            progress: extern fn(context: *mut std::os::raw::c_void,
                                                total: std::os::raw::c_uint,
                                                current: std::os::raw::c_uint,
                                                percent: std::os::raw::c_double,
                                                tick: std::os::raw::c_uint)) -> std::os::raw::c_int {
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

    match sync(c.0.get_api_token(),
               &n,
               main_key,
               hmac_key,
               tweak_key,
               id,
               &mut |total, current, new, progress_percent, tick| {
                   let c_total: std::os::raw::c_uint = total;
                   let c_current: std::os::raw::c_uint = current;
                   let c_new: std::os::raw::c_uint = new;
                   let c_percent: std::os::raw::c_double = progress_percent;
                   let c_tick: std::os::raw::c_uint =  if tick { 1 } else { 0 };
                   progress(context, c_total, c_current, c_percent, c_tick);
               }) {
        Ok(_) => 0,
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        }
    }
}

/// Start a restore for the folder ID
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
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using sddk_free_error()
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///      0: success
///
/// # Examples
///
/// ```c
/// void *context; // arbitrary, can be anything you want access to in the callback
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// if (0 != sddk_restore(&context, &state, &error, "02c0dc9c-6217-407b-a3ef-0d7ac5f288b1", 7, "/path/to/destination", &fp)) {
///     printf("Failed to restore");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Restore successful");
/// }
///
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_restore(context: *mut std::os::raw::c_void,
                               state: *mut SDDKState,
                               mut error: *mut *mut SDDKError,
                               name: *const std::os::raw::c_char,
                               folder_id: std::os::raw::c_uint,
                               destination: *const std::os::raw::c_char,
                               progress: extern fn(context: *mut std::os::raw::c_void,
                                                   total: std::os::raw::c_uint,
                                                   current: std::os::raw::c_uint,
                                                   percent: std::os::raw::c_double,
                                                   tick: std::os::raw::c_uint)) -> std::os::raw::c_int {
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
    let p = PathBuf::from(d);

    let main_key = (*c).0.get_main_key();
    let hmac_key = (*c).0.get_hmac_key();
    let tweak_key = (*c).0.get_tweak_key();

    let id: u32 = folder_id as u32;

    match restore(c.0.get_api_token(),
                  &n,
                  main_key,
                  id,
                  p,
                  &mut |total, current, new, progress_percent, tick| {
                      let c_total: std::os::raw::c_uint = total;
                      let c_current: std::os::raw::c_uint = current;
                      let c_percent: std::os::raw::c_double = progress_percent;
                      let c_new: std::os::raw::c_uint = new;
                      let c_tick: std::os::raw::c_uint =  if tick { 1 } else { 0 };
                      progress(context, c_total, c_current, c_percent, c_tick);
                  }) {
        Ok(_) => 0,
        Err(e) => {
            let c_err = SDDKError::from(e);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        }
    }
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
    let _ = unsafe { CString::from_raw(folder.name as *mut std::os::raw::c_char) };
    let _ = unsafe { CString::from_raw(folder.path as *mut std::os::raw::c_char) };
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
pub extern "C" fn sddk_free_folders(folders: *mut *mut SDDKFolder,
                                    length: u64) {
    assert!(!folders.is_null());
    let l = length as usize;

    let folders: Vec<SDDKFolder> = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(*folders, l)).into_vec() };
    for folder in folders {
        let _ = unsafe { CString::from_raw(folder.name as *mut std::os::raw::c_char) };
        let _ = unsafe { CString::from_raw(folder.path as *mut std::os::raw::c_char) };
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
pub extern "C" fn sddk_free_sync_sessions(sessions: *mut *mut SDDKSyncSession,
                                          length: u64) {
    assert!(!sessions.is_null());
    let l = length as usize;
    let sessions: Vec<SDDKSyncSession> = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(*sessions, l)).into_vec() };
    for session in sessions {
        let _ = unsafe { CString::from_raw(session.name as *mut std::os::raw::c_char) };
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
///     string: a pointer obtained from calling a function that returns a C string
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



/// Free a pointer to an SDDKError
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     error: a pointer to an SDDKError obtained from another call
///
///
///
/// # Examples
///
/// ```c
///sddk_free_error(&error);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_free_error(error: *mut *mut SDDKError) {
    assert!(!error.is_null());
    let e: Box<SDDKError> = unsafe { Box::from_raw(*error) };
    let _ = unsafe { CString::from_raw(e.message as *mut std::os::raw::c_char) };
}