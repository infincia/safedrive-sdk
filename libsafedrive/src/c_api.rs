#![allow(unused_variables)]
#![allow(unused_mut)]

use std;
use std::ffi::{CStr, CString};
use std::str;
use std::path::{Path, PathBuf};

use std::u64;
/// internal imports

use ::state::State;

use ::core::initialize;

use ::core::add_sync_folder;
use ::core::remove_sync_folder;
use ::core::get_sync_folder;
use ::core::get_sync_folders;

use ::core::get_sync_sessions;
use ::core::remove_sync_session;
use ::core::clean_sync_sessions;
use ::core::sync;
use ::core::restore;
use ::core::load_keys;
use ::core::login;

use ::constants::Configuration;

use ::models::{RegisteredFolder, AccountStatus, AccountDetails, Notification, SyncCleaningSchedule};

use ::session::SyncSession;

use ::core::get_unique_client_id;
use ::core::generate_unique_client_id;

use ::core::get_account_status;
use ::core::get_account_details;

use ::core::send_error_report;

use ::error::SDError;

/// exports

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
pub struct SDDKLogLine {
    pub line: *const std::os::raw::c_char,
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKNotification {
    pub title: *const std::os::raw::c_char,
    pub message: *const std::os::raw::c_char,
}

impl From<Notification> for SDDKNotification {
    fn from(notification: Notification) -> SDDKNotification {
        SDDKNotification {
            title: CString::new(notification.title.as_str()).unwrap().into_raw(),
            message: CString::new(notification.message.as_str()).unwrap().into_raw(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKAccountStatus {
    pub status: *const std::os::raw::c_char,
    pub host: *const std::os::raw::c_char,
    pub port: u16,
    pub user_name: *const std::os::raw::c_char,
    pub time: *const std::os::raw::c_ulonglong,
}

impl From<AccountStatus> for SDDKAccountStatus {
    fn from(status: AccountStatus) -> SDDKAccountStatus {
        SDDKAccountStatus {
            status: {
                if let Some(s) = status.status {
                    CString::new(s.as_str()).unwrap().into_raw()
                } else {
                    std::ptr::null()
                }
            },
            host: CString::new(status.host.as_str()).unwrap().into_raw(),
            port: status.port,
            user_name: CString::new(status.userName.as_str()).unwrap().into_raw(),
            time: {
                if let Some(t) = status.time {
                    let b = Box::new(t);
                    let ptr = Box::into_raw(b);

                    ptr
                } else {
                    std::ptr::null()
                }
            },
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKAccountDetails {
    pub assigned_storage: u64,
    pub used_storage: u64,
    pub low_free_space_threshold: i64,
    pub expiration_date: u64,
    pub notifications:  *const SDDKNotification,
    pub notification_count: i32,

}

impl From<AccountDetails> for SDDKAccountDetails {
    fn from(details: AccountDetails) -> SDDKAccountDetails {
        let (c_notifications, c_notifications_count) = match details.notifications {
            Some(notifications) => {
                let cn = notifications.into_iter().map(|notification| {
                    SDDKNotification::from(notification)
                }).collect::<Vec<SDDKNotification>>();

                let mut b = cn.into_boxed_slice();
                let ptr = b.as_mut_ptr();
                let len = b.len();
                std::mem::forget(b);

                (Some(ptr), len as i32)
            },
            None => (None, -1),
        };




        let mut c_details = SDDKAccountDetails {
            assigned_storage: details.assignedStorage,
            used_storage: details.usedStorage,
            low_free_space_threshold: details.lowFreeStorageThreshold,
            expiration_date: details.expirationDate,
            notifications: std::ptr::null(),
            notification_count: c_notifications_count as i32,
        };

        if let Some(c_n) = c_notifications {
            c_details.notifications = c_n;
        }

        c_details
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKFolder {
    pub id: u64,
    pub name: *const std::os::raw::c_char,
    pub path: *const std::os::raw::c_char,
    pub date: u64,
    pub encrypted: i8,
}

impl From<RegisteredFolder> for SDDKFolder {
    fn from(folder: RegisteredFolder) -> SDDKFolder {
        SDDKFolder {
            id: folder.id,
            name: CString::new(folder.folderName.as_str()).unwrap().into_raw(),
            path: CString::new(folder.folderPath.as_str()).unwrap().into_raw(),
            date: folder.addedDate,
            encrypted: {
                if folder.encrypted {
                    1
                } else {
                    0
                }

            },
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKSyncSession {
    pub name: *const std::os::raw::c_char,
    pub size: u64,
    pub date: u64,
    pub folder_id: u64,
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
    SyncAlreadyInProgress = 0x000E,
    RestoreAlreadyInProgress = 0x000F,
    ExceededRetries = 0x0010,
    KeychainError = 0x0011,
    BlockUnreadable = 0x0012,
    SessionUnreadable = 0x0013,
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
            SDError::KeychainError(_) => {
                SDDKErrorType::KeychainError
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
                SDDKErrorType::BlockMissing
            },
            SDError::SessionMissing => {
                SDDKErrorType::SessionMissing
            },
            SDError::BlockUnreadable => {
                SDDKErrorType::BlockUnreadable
            },
            SDError::SessionUnreadable => {
                SDDKErrorType::SessionUnreadable
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
            SDError::SyncAlreadyInProgress => {
                SDDKErrorType::SyncAlreadyInProgress
            },
            SDError::RestoreAlreadyInProgress => {
                SDDKErrorType::RestoreAlreadyInProgress
            },
            SDError::ExceededRetries(_) => {
                SDDKErrorType::ExceededRetries
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

#[derive(Debug)]
#[repr(C)]
pub enum SDDKSyncCleaningSchedule {
    /// Clean sync sessions on a structured schedule.
    ///
    /// The goal is to keep frequent sessions for the recent past, and retain fewer of them as they
    /// age. Will clean the sessions that would be considered redundant to keep around. Where possible
    /// it will keep the oldest session available within a given time period.
    ///
    /// Within a 30 day period, it will keep:
    ///
    /// Hourly: at most 1 per hour for the past 24 hours
    /// Daily:  at most 1 per day for the earlier 7 days
    /// Weekly: at most 1 per week for the earlier 23 days
    ///
    /// Beyond that 30 day period, it will keep at most 1 per month
    ///
    ///
    /// Example:
    ///
    /// For a new account created on January 1st, 2017, that had created 1 session at the start of
    /// each hour (xx:00:00) for the past 3 months, after running a cleaning job on
    /// April 1st at 00:30:00, the user would retain:
    ///
    /// Hourly: 1 session for each of the past 24 hours
    ///
    ///         1 - March 31st at 01:00:00
    ///
    ///         ...
    ///
    ///         24 - April 1st at 00:00:00
    ///
    /// Daily: 1 session for each of March 25th - 31st
    ///        1 - March 25th at 00:00:00
    ///        2 - March 26th at 00:00:00
    ///        3 - March 27th at 00:00:00
    ///        4 - March 28th at 00:00:00
    ///        5 - March 29th at 00:00:00
    ///        6 - March 30th at 00:00:00
    ///        7 - March 31st at 00:00:00
    ///
    /// Weekly: 1 session within each 7 day period between March 1st at 00:00:00 and March 25th at 00:00:00.
    ///        1 - March 1st at 00:00:00
    ///        2 - March 8th at 00:00:00
    ///        3 - March 15th at 00:00:00
    ///        4 - March 22nd at 00:00:00
    ///
    /// Monthly: 1 session for each of January and February
    ///        1 - January 1st at 00:00:00
    ///        2 - February 1st at 00:00:00
    ///
    ///
    ///
    SDDKSyncCleaningScheduleAuto,

    /// Clean all sessions older than an exact date
    ///
    SDDKSyncCleaningScheduleExactDateRFC3339,
    SDDKSyncCleaningScheduleExactDateRFC2822,

    /// The remaining schedules will clean all sync sessions older than a specific time period
    ///
    SDDKSyncCleaningScheduleAll,             // current time, should delete all of them
    SDDKSyncCleaningScheduleBeforeToday,     // today at 00:00
    SDDKSyncCleaningScheduleBeforeThisWeek,  // the first day of this week at 00:00
    SDDKSyncCleaningScheduleBeforeThisMonth, // the first day of this month at 00:00
    SDDKSyncCleaningScheduleBeforeThisYear,  // the first day of this year at 00:00
    SDDKSyncCleaningScheduleOneDay,          // 24 hours
    SDDKSyncCleaningScheduleOneWeek,         // 7 days
    SDDKSyncCleaningScheduleOneMonth,        // 30 days
    SDDKSyncCleaningScheduleOneYear,         // 365 days
}

impl SDDKSyncCleaningSchedule {
    fn to_schedule(&self, date: Option<String>) -> SyncCleaningSchedule {
        match *self {
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleAuto => {
                SyncCleaningSchedule::Auto
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleExactDateRFC3339 => {
                SyncCleaningSchedule::ExactDateRFC3339 { date: date.unwrap() }
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleExactDateRFC2822 => {
                SyncCleaningSchedule::ExactDateRFC2822 { date: date.unwrap() }
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleAll => {
                SyncCleaningSchedule::All
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleBeforeToday => {
                SyncCleaningSchedule::BeforeToday
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleBeforeThisWeek => {
                SyncCleaningSchedule::BeforeThisWeek
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleBeforeThisMonth => {
                SyncCleaningSchedule::BeforeThisMonth
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleBeforeThisYear => {
                SyncCleaningSchedule::BeforeThisYear
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleOneDay => {
                SyncCleaningSchedule::OneDay
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleOneWeek => {
                SyncCleaningSchedule::OneWeek
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleOneMonth => {
                SyncCleaningSchedule::OneMonth
            },
            SDDKSyncCleaningSchedule::SDDKSyncCleaningScheduleOneYear => {
                SyncCleaningSchedule::OneYear
            },
        }
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
///     client_version: a stack-allocated, NULL-terminated string representing the name and version of the current app
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
/// SDDKState *state = sddk_initialize("/home/user/.safedrive/", "12345", "SafeDrive 0.9", SDDKConfigurationProduction)
/// // do something with SDDKState here, store it somewhere, then free it later on
/// sddk_free_state(&state);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_initialize(client_version: *const std::os::raw::c_char,
                                  operating_system: *const std::os::raw::c_char,
                                  language_code: *const std::os::raw::c_char,
                                  config: SDDKConfiguration) -> *mut SDDKState {

    let cvs: &CStr = unsafe {
        assert!(!client_version.is_null());
        CStr::from_ptr(client_version)
    };

    let cv: String = match cvs.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

    let oss: &CStr = unsafe {
        assert!(!operating_system.is_null());
        CStr::from_ptr(operating_system)
    };

    let os: String = match oss.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

    let c_language_code: &CStr = unsafe {
        assert!(!language_code.is_null());
        CStr::from_ptr(language_code)
    };

    let langc: String = match c_language_code.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

    let c = match config {
        SDDKConfiguration::SDDKConfigurationProduction => Configuration::Production,
        SDDKConfiguration::SDDKConfigurationStaging => Configuration::Staging,
    };

    initialize(&cv, &os, &langc, c);

    let sstate = State::new();
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
/// SDDKError *uid_error = NULL;
/// SDDKAccountStatus * status = NULL;
/// char* unique_client_id = NULL;
///
/// char[] app_directory = "/home/user/.safedrive";
/// char[] user = "user@safedrive.io";
/// char[] password = "password";
///
/// if (0 != ssdk_get_unique_client_id(&app_directory, &unique_client_id, &uid_error)) {
///     printf("getting unique client id failed");
///     //do something with uid_error here, then free it
///     sddk_free_error(&uid_error);
///     return; // can't continue without one
/// }
///
/// SDDKError *login_error = NULL;
///
/// if (0 != sddk_login(&state, &app_directory, &user, &password, &status, &login_error)) {
///     printf("Login failed");
///     // do something with error here, then free it
///     sddk_free_error(&error);
///
///     // also free unique_client_id
///     sddk_free_string(&unique_client_id);
/// }
/// else {
///     printf("Login successful");
///     // do something with status here, then free it
///
///     sddk_free_account_status(&status);
///     // also free unique_client_id
///     sddk_free_string(&unique_client_id);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_login(state: *mut SDDKState,
                             local_storage_path: *const std::os::raw::c_char,
                             unique_client_id: *const std::os::raw::c_char,
                             username: *const std::os::raw::c_char,
                             password:  *const std::os::raw::c_char,
                             mut status: *mut *mut SDDKAccountStatus,
                             mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!state.is_null()); assert!(!username.is_null()); assert!(!password.is_null()); &mut * state };

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

    match login(&uid, &storage_path, &un, &pa) {
        Ok((token, account_status)) => {
            c.0.set_account(un.to_owned(), pa.to_owned());
            c.0.set_api_token(token);
            c.0.set_unique_client_id(uid);
            let c_s = SDDKAccountStatus::from(account_status);

            let b = Box::new(c_s);
            let ptr = Box::into_raw(b);

            unsafe {
                *status = ptr;
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
        /// call back to C to store phrase
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

/// Get the current unique client ID and store it in the `unique_client_id` parameter
///
/// Will return a failure code if there is no unique client ID available
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
/// SDDKError *error = NULL;
///
/// if (0 != sddk_get_unique_client_id(&unique_client_id, &error)) {
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
pub extern "C" fn sddk_get_unique_client_id(local_storage_path: *const std::os::raw::c_char,
                                            mut unique_client_id: *mut *mut std::os::raw::c_char,
                                            mut error: *mut *mut SDDKError) -> std::os::raw::c_int {

    let lstorage: &CStr = unsafe {
        assert!(!local_storage_path.is_null());
        CStr::from_ptr(local_storage_path)
    };

    let storage_directory: String = match lstorage.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };

    let storage_path = Path::new(&storage_directory);

    match get_unique_client_id(storage_path) {
        Ok(uid) => {
            unsafe {
                *unique_client_id = CString::new(uid).expect("Failed to get unique client id").into_raw();
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



/// Generate a unique client ID and store it in the `unique_client_id` parameter
///
/// Function cannot fail so has no return value for failure or error parameter.
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
///     unique_client_id: an uninitialized pointer that will be allocated and initialized when the function
///            returns
///
///            must be freed by the caller using sddk_free_string()
///
/// Return:
///
///      0+: length of unique client ID string as bytes/chars
///
///
/// # Examples
///
/// ```c
/// char * unique_client_id;
///
/// sddk_generate_unique_client_id(&unique_client_id);
/// printf("Generated unique client id");
///
/// // do something with unique_client_id here, then free it
/// sddk_free_string(&unique_client_id);
///
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_generate_unique_client_id(mut unique_client_id: *mut *mut std::os::raw::c_char) -> std::os::raw::c_uint {

    let uid = generate_unique_client_id();
    let length = uid.len();

    unsafe {
        *unique_client_id = CString::new(uid).expect("Failed to get unique client id").into_raw();
    }

    length as u32
}

/// Get account status from the SafeDrive server
///
/// The caller does not own the memory pointed to by `status` or `error` after this function returns,
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
///     status: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was 0
///
///            must be freed by the caller using sddk_free_account_status()
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
///     0+: success
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
/// SDDKAccountStatus * status = NULL;
///
/// if (0 != sddk_get_account_status(&state, &status, &error)) {
///     printf("Failed to get account status");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Got account status");
///     // do something with status here, then free it
///     sddk_free_account_status(&status);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_account_status(state: *mut SDDKState,
                                          mut status: *mut *mut SDDKAccountStatus,
                                          mut error: *mut *mut SDDKError) -> i8 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let s = match get_account_status(c.0.get_api_token()) {
        Ok(s) => s,
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

    let c_s = SDDKAccountStatus::from(s);

    let b = Box::new(c_s);
    let ptr = Box::into_raw(b);

    unsafe {
        *status = ptr;
    }

    0
}


/// Get account details from the SafeDrive server
///
/// The caller does not own the memory pointed to by `details` or `error` after this function returns,
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
///     details: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was 0
///
///            must be freed by the caller using sddk_free_account_details()
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
///     0+: success
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
/// SDDKAccountDetails * details = NULL;
///
/// if (0 != sddk_get_account_details(&state, &details, &error)) {
///     printf("Failed to get account details");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Got account details");
///     // do something with details here, then free it
///     sddk_free_account_details(&details);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_account_details(state: *mut SDDKState,
                                          mut details: *mut *mut SDDKAccountDetails,
                                          mut error: *mut *mut SDDKError) -> i8 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let d = match get_account_details(c.0.get_api_token()) {
        Ok(d) => d,
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

    let c_d = SDDKAccountDetails::from(d);

    let b = Box::new(c_d);
    let ptr = Box::into_raw(b);

    unsafe {
        *details = ptr;
    }

    0
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
                                       mut error: *mut *mut SDDKError) -> std::os::raw::c_longlong {
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
        Ok(folder_id) => folder_id as i64,
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
///     folder_id: a stack-allocated, unsigned 64-bit integer representing the registered folder ID
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
                                          folder_id: std::os::raw::c_ulonglong,
                                          mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let id = folder_id as u64;


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
///     folder_id: a stack-allocated, unsigned 64-bit integer representing a registered folder ID
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
                                       folder_id: std::os::raw::c_ulonglong,
                                       mut folder: *mut *mut SDDKFolder,
                                       mut error: *mut *mut SDDKError) -> i8 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };
    let id = folder_id as u64;

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


/// Remove a sync session
///
/// Will return a failure code and set the error parameter if for any reason the session cannot be removed
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///     session_id: an unsigned 64-bit integer representing the session ID
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
/// if (0 != sddk_remove_sync_session(&state, 7, &error)) {
///     printf("Failed to remove session");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Removed session");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_remove_sync_session(state: *mut SDDKState,
                                           session_id: std::os::raw::c_ulonglong,
                                           mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let id = session_id as u64;


    match remove_sync_session(c.0.get_api_token(), id) {
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

/// Clean sync sessions according to the provided schedule type
///
/// Will return a failure code and set the error parameter if for any reason the sessions cannot be
/// cleaned
///
///
/// Parameters:
///
///     state: an opaque pointer obtained from calling sddk_initialize()
///
///  schedule: an SDDKSyncCleaningSchedule variant
///
///      date: the date string used for SDDKSyncCleaningSchedule.ExactDateRFC* variants. Should be
///            null for other enum variants.
///
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
/// if (0 != sddk_clean_sync_sessions(&state, SDDKSyncCleaningSchedule.All, &error)) {
///     printf("Failed to remove sessions");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Removed sessions");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_clean_sync_sessions(state: *mut SDDKState,
                                           schedule: SDDKSyncCleaningSchedule,
                                           date: *const std::os::raw::c_char,
                                           mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };


    let d: Option<String> = unsafe {
        if date.is_null() {
            None
        } else {
            let c_date: &CStr = CStr::from_ptr(date);
            let d: String = match c_date.to_str() {
                Ok(s) => s.to_owned(),
                Err(e) => { panic!("string is not valid UTF-8: {}", e) },
            };

            Some(d)
        }
    };

    let sch = schedule.to_schedule(d);

    match clean_sync_sessions(c.0.get_api_token(), sch) {
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
///     folder_id: a stack-allocated, unsigned 64-bit integer representing a registered folder ID
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
                            folder_id: std::os::raw::c_ulonglong,
                            progress: extern fn(context: *mut std::os::raw::c_void,
                                                total: std::os::raw::c_ulonglong,
                                                current: std::os::raw::c_ulonglong,
                                                new: std::os::raw::c_ulonglong,
                                                percent: std::os::raw::c_double,
                                                tick: std::os::raw::c_uint,
                                                message: *const std::os::raw::c_char)) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };
    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = match c_name.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => { panic!("string is not valid UTF-8: {}", e) },
    };


    let main_key = (*c).0.get_main_key();
    let hmac_key = (*c).0.get_hmac_key();
    let tweak_key = (*c).0.get_tweak_key();

    let id = folder_id as u64;

    match sync(c.0.get_api_token(),
               &n,
               main_key,
               hmac_key,
               tweak_key,
               id,
               &mut |total, current, new, progress_percent, tick, message| {
                   let c_total: std::os::raw::c_ulonglong = total;
                   let c_current: std::os::raw::c_ulonglong = current;
                   let c_new: std::os::raw::c_ulonglong = new;
                   let c_percent: std::os::raw::c_double = progress_percent;
                   let c_tick: std::os::raw::c_uint =  if tick { 1 } else { 0 };
                   let c_message = CString::new(message).expect("failed to get sync message");
                   progress(context, c_total, c_current, c_new, c_percent, c_tick, c_message.as_ptr());
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
///     folder_id: a stack-allocated, unsigned 64-bit integer representing a registered folder ID
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
                               folder_id: std::os::raw::c_ulonglong,
                               destination: *const std::os::raw::c_char,
                               session_size: std::os::raw::c_ulonglong,
                               progress: extern fn(context: *mut std::os::raw::c_void,
                                                   total: std::os::raw::c_ulonglong,
                                                   current: std::os::raw::c_ulonglong,
                                                   new: std::os::raw::c_ulonglong,
                                                   percent: std::os::raw::c_double,
                                                   tick: std::os::raw::c_uint,
                                                   message: *const std::os::raw::c_char)) -> std::os::raw::c_int {
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

    let id = folder_id as u64;
    let ses_size = session_size as u64;


    match restore(c.0.get_api_token(),
                  &n,
                  main_key,
                  id,
                  p,
                  ses_size,
                  &mut |total, current, new, progress_percent, tick, message| {
                      let c_total: std::os::raw::c_ulonglong = total;
                      let c_current: std::os::raw::c_ulonglong = current;
                      let c_percent: std::os::raw::c_double = progress_percent;
                      let c_new: std::os::raw::c_ulonglong = new;
                      let c_tick: std::os::raw::c_uint =  if tick { 1 } else { 0 };
                      let c_message = CString::new(message).expect("failed to get sync message");
                      progress(context, c_total, c_current, c_new, c_percent, c_tick, c_message.as_ptr());
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



/// Report an error to the SafeDrive server
///
/// Will return a failure code if the parameters are invalid or the request could not be completed
///
///
/// Parameters:
///
///     unique_client_id: a stack-allocated, NULL terminated string representing the current UCID
///
///     description: a stack-allocated, NULL terminated string representing the error description
///
///     context: a stack-allocated, NULL terminated string representing any context that should be
///              included in the error report
///
///     log: a pointer to an array of SDDKLogLine structs, caller retains ownership of the memory
///          and must free it after use
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
/// char[] unique_client_id = "1234";
///
/// SDDKError *error = NULL;
///
///
/// if (0 != sddk_report_error(&unique_client_id, &error)) {
///     printf("Failed to report error");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Error reported");
///
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_report_error(client_version: *const std::os::raw::c_char,
                                    operating_system: *const std::os::raw::c_char,
                                    unique_client_id: *const std::os::raw::c_char,
                                    description: *const std::os::raw::c_char,
                                    context: *const std::os::raw::c_char,
                                    log: *const SDDKLogLine,
                                    logline_count: u64,
                                    mut error: *mut *mut SDDKError) -> std::os::raw::c_int {

    let ucid: String = unsafe {
        assert!(!unique_client_id.is_null());

        let c_ucid: &CStr = CStr::from_ptr(unique_client_id);

        let ucid: String = match c_ucid.to_str() {
            Ok(s) => s.to_owned(),
            Err(e) => { panic!("string is not valid UTF-8: {}", e) },
        };

        ucid
    };

    let desc: String = unsafe {
        assert!(!description.is_null());

        let c_desc: &CStr = CStr::from_ptr(description);

        let desc: String = match c_desc.to_str() {
            Ok(s) => s.to_owned(),
            Err(e) => { panic!("string is not valid UTF-8: {}", e) },
        };

        desc
    };

    let cont: String = unsafe {
        assert!(!context.is_null());

        let c_cont: &CStr = CStr::from_ptr(context);

        let cont: String = match c_cont.to_str() {
            Ok(s) => s.to_owned(),
            Err(e) => { panic!("string is not valid UTF-8: {}", e) },
        };

        cont
    };

    let rlog = unsafe {
        assert!(!log.is_null());
        let loglines = std::slice::from_raw_parts(log, logline_count as usize);

        let mut rlv: Vec<&str > = Vec::new();

        for line in loglines {
            assert!(!line.line.is_null());

            let c_line: &CStr = CStr::from_ptr(line.line);

            let l = match c_line.to_str() {
                Ok(s) => s,
                Err(e) => { panic!("string is not valid UTF-8: {}", e) },
            };

            rlv.push(l);
        }

        rlv
    };

    let ver: Option<String> = unsafe {
        if client_version.is_null() {
            None
        } else {
            let c_client_version: &CStr = CStr::from_ptr(client_version);

            let ver: String = match c_client_version.to_str() {
                Ok(s) => s.to_owned(),
                Err(e) => { panic!("string is not valid UTF-8: {}", e) },
            };

            Some(ver)
        }
    };

    let os: Option<String> = unsafe {
        if operating_system.is_null() {
            None
        } else {
            let c_operating_system: &CStr = CStr::from_ptr(operating_system);

            let os: String = match c_operating_system.to_str() {
                Ok(s) => s.to_owned(),
                Err(e) => { panic!("string is not valid UTF-8: {}", e) },
            };

            Some(os)
        }
    };


    match send_error_report(ver, os, &ucid, &desc, &cont, &rlog) {
        Ok(()) => {
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

/// Free a pointer to an SDDKAccountStatus
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     status: a pointer to an SDDKStatus obtained from another call
///
///
///
/// # Examples
///
/// ```c
///sddk_free_account_status(&status);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_free_account_status(status: *mut *mut SDDKAccountStatus) {
    assert!(!status.is_null());
    let s: Box<SDDKAccountStatus> = unsafe { Box::from_raw(*status) };
    if !s.status.is_null() {
        let _ = unsafe {
            CString::from_raw(s.status as *mut std::os::raw::c_char)
        };
    }
    if !s.time.is_null() {
        let _ = unsafe {
            Box::from_raw(s.time as *mut std::os::raw::c_ulonglong)
        };
    }
}

/// Free a pointer to an SDDKAccountDetails
///
/// Note: This is *not* the same as calling free() in C, they are not interchangeable
///
/// Parameters:
///
///     details: a pointer to an SDDKAccountDetails obtained from another call
///
///
///
/// # Examples
///
/// ```c
///sddk_free_account_details(&details);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_free_account_details(details: *mut *mut SDDKAccountDetails) {
    assert!(!details.is_null());
    let d: Box<SDDKAccountDetails> = unsafe { Box::from_raw(*details) };
    if !d.notifications.is_null() {
        let _ = unsafe {
            let notifications: Vec<SDDKNotification> = Box::from_raw(std::slice::from_raw_parts_mut(d.notifications as *mut SDDKNotification, d.notification_count as usize)).into_vec();
            for notification in notifications {
                let _ = CString::from_raw(notification.title as *mut std::os::raw::c_char);
                let _ = CString::from_raw(notification.message as *mut std::os::raw::c_char);

            }
        };
    }
}