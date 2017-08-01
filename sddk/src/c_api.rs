#![allow(unused_variables)]
#![allow(unused_mut)]

use std;
use std::ffi::{CStr, CString, OsString};
use std::str;
use std::path::{PathBuf};

use std::u64;
/// internal imports

use state::State;

use core::initialize;

use core::add_sync_folder;
use core::update_sync_folder;
use core::remove_sync_folder;
use core::get_sync_folder;
use core::get_sync_folders;
use core::has_conflicting_folder;

use core::log;

use core::get_sync_sessions;
use core::remove_sync_session;
use core::clean_sync_sessions;
use sync::sync;
use restore::restore;
use core::load_keys;
use core::login;
use sync_state::cancel_sync_task;

use constants::Configuration;

use models::{RegisteredFolder, AccountStatus, AccountState, AccountDetails, Notification, SyncCleaningSchedule, SoftwareClient};

use keychain::KeychainService;
use core::get_keychain_item;
use core::set_keychain_item;
use core::delete_keychain_item;

use session::SyncSession;

use core::generate_unique_client_id;

use core::get_account_status;
use core::get_account_details;

use core::remove_software_client;

use core::get_software_clients;

use core::send_error_report;

use core::remote_mkdir;
use core::remote_mv;
use core::remote_rmdir;
use core::remote_rm;

use error::SDError;

/// exports
#[derive(Debug)]
#[repr(C)]
pub struct SDDKState(State);

#[derive(Debug)]
#[repr(C)]
pub enum SDDKConfiguration {
    Production,
    Staging,
}


#[derive(Debug)]
#[repr(C)]
pub enum SDDKLogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
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
pub enum SDDKAccountState {
    Unknown,
    Active,
    Trial,
    TrialExpired,
    Expired,
    Locked,
    ResetPassword,
    PendingCreation,
}

impl From<AccountState> for SDDKAccountState {
    fn from(e: AccountState) -> SDDKAccountState {
        match e {
            AccountState::Unknown => SDDKAccountState::Unknown,
            AccountState::Active => SDDKAccountState::Active,
            AccountState::Trial => SDDKAccountState::Trial,
            AccountState::TrialExpired => SDDKAccountState::TrialExpired,
            AccountState::Expired => SDDKAccountState::Expired,
            AccountState::Locked => SDDKAccountState::Locked,
            AccountState::ResetPassword => SDDKAccountState::ResetPassword,
            AccountState::PendingCreation => SDDKAccountState::PendingCreation,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKAccountStatus {
    pub state: SDDKAccountState,
    pub host: *const std::os::raw::c_char,
    pub port: u16,
    pub user_name: *const std::os::raw::c_char,
    pub time: *const std::os::raw::c_ulonglong,
}

impl From<AccountStatus> for SDDKAccountStatus {
    fn from(status: AccountStatus) -> SDDKAccountStatus {
        SDDKAccountStatus {
            state: {
                if let Some(state) = status.state {
                    SDDKAccountState::from(state)
                } else {
                    SDDKAccountState::Unknown
                }
            },
            host: CString::new(status.host.as_str()).unwrap().into_raw(),
            port: status.port,
            user_name: CString::new(status.userName.as_str()).unwrap().into_raw(),
            time: {
                if let Some(t) = status.time {
                    let b = Box::new(t);
                    Box::into_raw(b)
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
        /*let (c_notifications, c_notifications_count) = match details.notifications {
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
        };*/


        /*if let Some(c_n) = c_notifications {
            c_details.notifications = c_n;
        }*/

        SDDKAccountDetails {
            assigned_storage: details.assignedStorage,
            used_storage: details.usedStorage,
            low_free_space_threshold: details.lowFreeStorageThreshold,
            expiration_date: details.expirationDate,
            notifications: std::ptr::null(),
            notification_count: 0 as i32,
        }
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
    pub syncing: i8,
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
            syncing: {
                if folder.syncing {
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
    pub session_id: u64,
}

impl From<SyncSession> for SDDKSyncSession {
    fn from(session: SyncSession) -> SDDKSyncSession {
        SDDKSyncSession {
            folder_id: session.folder_id.unwrap(),
            size: session.size.unwrap(),
            name: CString::new(session.name.as_str()).unwrap().into_raw(),
            date: session.time.unwrap(),
            session_id: session.id.unwrap(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SDDKSoftwareClient {
    pub unique_client_id: *const std::os::raw::c_char,
    pub operating_system: *const std::os::raw::c_char,
    pub language: *const std::os::raw::c_char,
}

impl From<SoftwareClient> for SDDKSoftwareClient {
    fn from(client: SoftwareClient) -> SDDKSoftwareClient {
        SDDKSoftwareClient {
            unique_client_id: CString::new(client.uniqueId.as_str()).unwrap().into_raw(),
            operating_system: CString::new(client.operatingSystem.as_str()).unwrap().into_raw(),
            language: CString::new(client.language.as_str()).unwrap().into_raw(),
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
    ServiceUnavailable = 0x0014,
    Cancelled = 0x0015,
    FolderMissing = 0x0016,
    KeyCorrupted = 0x0017,
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
            SDError::Internal(_) => SDDKErrorType::Internal,
            SDError::IO(_) => SDDKErrorType::IO,
            SDError::KeychainError(_) => SDDKErrorType::KeychainError,
            SDError::RequestFailure(_) => SDDKErrorType::RequestFailure,
            SDError::NetworkFailure(_) => SDDKErrorType::NetworkFailure,
            SDError::Conflict(_) => SDDKErrorType::Conflict,
            SDError::BlockMissing => SDDKErrorType::BlockMissing,
            SDError::SessionMissing => SDDKErrorType::SessionMissing,
            SDError::BlockUnreadable => SDDKErrorType::BlockUnreadable,
            SDError::SessionUnreadable => SDDKErrorType::SessionUnreadable,
            SDError::RecoveryPhraseIncorrect => SDDKErrorType::RecoveryPhraseIncorrect,
            SDError::KeyCorrupted => SDDKErrorType::KeyCorrupted,
            SDError::InsufficientFreeSpace => SDDKErrorType::InsufficientFreeSpace,
            SDError::Authentication => SDDKErrorType::Authentication,
            SDError::UnicodeError => SDDKErrorType::UnicodeError,
            SDError::TokenExpired => SDDKErrorType::TokenExpired,
            SDError::CryptoError(_) => SDDKErrorType::CryptoError,
            SDError::SyncAlreadyInProgress => SDDKErrorType::SyncAlreadyInProgress,
            SDError::RestoreAlreadyInProgress => SDDKErrorType::RestoreAlreadyInProgress,
            SDError::ExceededRetries(_) => SDDKErrorType::ExceededRetries,
            SDError::ServiceUnavailable => SDDKErrorType::ServiceUnavailable,
            SDError::Cancelled => SDDKErrorType::Cancelled,
            SDError::FolderMissing => SDDKErrorType::FolderMissing,
        };
        SDDKError {
            error_type: error_type,
            message: CString::new(format!("{}", e)).unwrap().into_raw(),
        }
    }
}

impl From<str::Utf8Error> for SDDKError {
    fn from(e: str::Utf8Error) -> Self {
        SDDKError {
            error_type: SDDKErrorType::UnicodeError,
            message: CString::new(format!("{}", e)).unwrap().into_raw(),
        }
    }
}


impl From<String> for SDDKError {
    fn from(s: String) -> Self {
        SDDKError {
            error_type: SDDKErrorType::Internal,
            message: CString::new(s).unwrap().into_raw(),
        }
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
    Auto,

    /// Clean all sessions older than an exact date
    ///
    ExactDateRFC3339,
    ExactDateRFC2822,

    /// The remaining schedules will clean all sync sessions older than a specific time period
    ///
    All,             // current time, should delete all of them
    BeforeToday,     // today at 00:00
    BeforeThisWeek,  // the first day of this week at 00:00
    BeforeThisMonth, // the first day of this month at 00:00
    BeforeThisYear,  // the first day of this year at 00:00
    OneDay,          // 24 hours
    OneWeek,         // 7 days
    OneMonth,        // 30 days
    OneYear,         // 365 days
}

impl SDDKSyncCleaningSchedule {
    fn to_schedule(&self, date: Option<String>) -> SyncCleaningSchedule {
        match *self {
            SDDKSyncCleaningSchedule::Auto => SyncCleaningSchedule::Auto,
            SDDKSyncCleaningSchedule::ExactDateRFC3339 => {
                SyncCleaningSchedule::ExactDateRFC3339 {
                    date: date.unwrap(),
                }
            },
            SDDKSyncCleaningSchedule::ExactDateRFC2822 => {
                SyncCleaningSchedule::ExactDateRFC2822 {
                    date: date.unwrap(),
                }
            },
            SDDKSyncCleaningSchedule::All => SyncCleaningSchedule::All,
            SDDKSyncCleaningSchedule::BeforeToday => SyncCleaningSchedule::BeforeToday,
            SDDKSyncCleaningSchedule::BeforeThisWeek => SyncCleaningSchedule::BeforeThisWeek,
            SDDKSyncCleaningSchedule::BeforeThisMonth => SyncCleaningSchedule::BeforeThisMonth,
            SDDKSyncCleaningSchedule::BeforeThisYear => SyncCleaningSchedule::BeforeThisYear,
            SDDKSyncCleaningSchedule::OneDay => SyncCleaningSchedule::OneDay,
            SDDKSyncCleaningSchedule::OneWeek => SyncCleaningSchedule::OneWeek,
            SDDKSyncCleaningSchedule::OneMonth => SyncCleaningSchedule::OneMonth,
            SDDKSyncCleaningSchedule::OneYear => SyncCleaningSchedule::OneYear,
        }
    }
}


/// Initialize the library, must be called before any other function.
///
/// If the application needs to switch users or the unique client ID changes, free the `SDDKState` and
/// call initialize again
///
/// Will assert non-null on `local_storage_path` and `unique_client_id`
///
/// Parameters:
///
///     Note: every parameter except `config` can be NULL, a sane default will be used in that case
///
///     `local_storage_path`: a NULL-terminated string representing the location the app can store settings for a user
///
///     `operating_system`: a NULL-terminated string representing the current OS name
///
///     `language_code`: a NULL-terminated string representing the current language/locale
///
///     `unique_client_id`: a NULL-terminated string representing the current unique client id
///
///     `client_version`: a NULL-terminated string representing the name and version of the current app
///
///     `config`: an enum variant of `SDDKConfiguration` which controls API endpoint and other things
///
///             use `SDDKConfigurationStaging` for the staging environment
///             use `SDDKConfigurationProduction` for the production environment
///
///     state: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was 0
///
///            must be freed by the caller using `sddk_free_state()`
///
///     error: an uninitialized pointer that will be allocated and initialized when the function
///            returns if the return value was -1
///
///            must be freed by the caller using `sddk_free_error()`
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
                                  config: SDDKConfiguration,
                                  local_storage_path: *const std::os::raw::c_char,
                                  log_level: SDDKLogLevel,
                                  mut state: *mut *mut SDDKState,
                                  mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
    let cv: String = match client_version.is_null() {
        true => "9999".to_owned(),
        false => {
            let cvs: &CStr = unsafe { CStr::from_ptr(client_version) };

            match cvs.to_str() {
                Ok(s) => s.to_owned(),
                Err(err) => panic!("string is not valid UTF-8: {}", err),
            }
        },
    };

    let os: String = match operating_system.is_null() {
        true => ::core::get_current_os().to_owned(),
        false => {
            let oss: &CStr = unsafe { CStr::from_ptr(operating_system) };

            match oss.to_str() {
                Ok(s) => s.to_owned(),
                Err(err) => panic!("string is not valid UTF-8: {}", err),
            }
        },
    };

    let langc: String = match language_code.is_null() {
        true => {
            "en_US".to_owned() //sane default if nothing was passed in, might want to check libc instead
        },
        false => {
            let c_language_code: &CStr = unsafe { CStr::from_ptr(language_code) };

            match c_language_code.to_str() {
                Ok(s) => s.to_owned(),
                Err(err) => panic!("string is not valid UTF-8: {}", err),
            }
        },
    };

    let c = match config {
        SDDKConfiguration::Production => Configuration::Production,
        SDDKConfiguration::Staging => Configuration::Staging,
    };

    let storage_path: PathBuf = match local_storage_path.is_null() {
        true => {
            //sane default if nothing was passed in,
            match ::core::get_app_directory(&c) {
                Ok(p) => p,
                Err(err) => panic!("string is not valid UTF-8: {}", err),
            }
        },
        false => {
            let lstorage: &CStr = unsafe { CStr::from_ptr(local_storage_path) };

            let s = match lstorage.to_str() {
                Ok(s) => s,
                Err(err) => panic!("string is not valid UTF-8: {}", err),
            };

            PathBuf::from(s)
        },
    };

    let level = match log_level {
        SDDKLogLevel::Error => ::log::LogLevelFilter::Error,
        SDDKLogLevel::Warn => ::log::LogLevelFilter::Warn,
        SDDKLogLevel::Info => ::log::LogLevelFilter::Info,
        SDDKLogLevel::Debug => ::log::LogLevelFilter::Debug,
        SDDKLogLevel::Trace => ::log::LogLevelFilter::Trace,
    };


    match initialize(&cv, true, &os, &langc, c, level, &storage_path) {
        Ok(()) => {

            let sstate = State::new();
            let c_state = SDDKState(sstate);

            let b = Box::new(c_state);
            let ptr = Box::into_raw(b);

            unsafe {
                *state = ptr;
            }
            0
        },
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Get SDK channel
///
/// Returned string must be passed back to the SDK to free it
///
/// # Examples
///
/// ```c
/// char *channel = sddk_get_channel();
/// // do something with channel here, store it somewhere, then free it later on
/// sddk_free_string(&channel);
/// ```
///
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_channel(mut channel: *mut *mut std::os::raw::c_char) {
    let c = ::core::get_channel();
    unsafe {
        *channel = CString::new(format!("{}", c).as_str()).unwrap().into_raw();
    }
}

/// Get SDK version
///
/// Returned string must be passed back to the SDK to free it
///
/// # Examples
///
/// ```c
/// char *version = sddk_get_version();
/// // do something with version here, store it somewhere, then free it later on
/// sddk_free_string(&version);
/// ```
///
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_version(mut version: *mut *mut std::os::raw::c_char) {
    let v = ::core::get_version();
    unsafe {
        *version = CString::new(v.as_str()).unwrap().into_raw();
    }
}

/// Get local app storage directory
///
/// Returned string must be passed back to the SDK to free it
///
///     `config`: an enum variant of `SDDKConfiguration` which is used to keep the two environments separate
///
///             use `SDDKConfigurationStaging` for the staging environment
///             use `SDDKConfigurationProduction` for the production environment
///
///`storage_path`: an uninitialized pointer that will be allocated and initialized when the function
///                returns if the return value was 0
///
///                must be freed by the caller using `sddk_free_string()`
///
///    `error`: an uninitialized pointer that will be allocated and initialized when the function
///             returns if the return value was -1
///
///             must be freed by the caller using `sddk_free_error()`
///
/// # Examples
///
/// ```c
/// char *storage_path = NULL;
/// SDDKError* error = NULL;
/// if (0 != sddk_get_app_directory(SDDKConfigurationStaging, &path, &error)) {
///     //do something with error here, then free it
///     sddk_free_error(&error);
/// } else {
///     //do something with storage_path here, store it somewhere, then free it later on
///     sddk_free_string(&storage_path);
/// }
/// ```
///
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_app_directory(config: SDDKConfiguration, mut storage_path: *mut *mut std::os::raw::c_char, mut error: *mut *mut SDDKError) -> std::os::raw::c_char {

    let c = match config {
        SDDKConfiguration::Production => Configuration::Production,
        SDDKConfiguration::Staging => Configuration::Staging,
    };

    let directory: PathBuf = match ::core::get_app_directory(&c) {
        Ok(p) => p,
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let osp: OsString = directory.into_os_string();

    match osp.into_string() {
        Ok(s) => {
            unsafe {
                *storage_path = CString::new(s).expect("Failed to get directory").into_raw();
            }
            0
        },
        Err(_) => {
            let c_err = SDDKError::from(SDError::UnicodeError);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        }
    }
}


/// Get a keychain item for a user/service pair, and store it in the `secret` parameter
///
/// Will return a failure code if the keychain item cannot be retrieved
///
/// The caller does not own the memory pointed to by `secret` after this function returns, it must
/// be returned and freed by the library.
///
/// As a result, any data that the caller wishes to retain must be copied out of the buffer before
/// it is freed.
///
///
/// Parameters:
///
///     `user`: a pointer to a NULL-terminated string representing the user account name
///
///  `service`: a pointer to a NULL-terminated string representing the service subdomain
///
///   `secret`: an uninitialized pointer that will be allocated and initialized when the function
///             returns if the return value was 0
///
///             must be freed by the caller using `sddk_free_string()`
///
///    `error`: an uninitialized pointer that will be allocated and initialized when the function
///             returns if the return value was -1
///
///             must be freed by the caller using `sddk_free_error()`
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
/// char * user = "user@safedrive.io";
/// char * service = "safedrive.io";
/// char * secret = NULL;
/// SDDKError *error = NULL;
///
/// if (0 != sddk_get_keychain_item(user, service, &secret, &error)) {
///     printf("Failed to get keychain item for user");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Got keychain item");
///     // do something with secret here, then free it
///     sddk_free_string(&secret);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_keychain_item(user: *const std::os::raw::c_char,
                                         service: *const std::os::raw::c_char,
                                         mut secret: *mut *mut std::os::raw::c_char,
                                         mut error: *mut *mut SDDKError) -> std::os::raw::c_int {

    if user.is_null() {

        let c_err = SDDKError::from(SDError::Internal(format!("user was null")));

        let b = Box::new(c_err);
        let ptr = Box::into_raw(b);

        unsafe {
            *error = ptr;
        }
        return -1;
    }

    if service.is_null() {

        let c_err = SDDKError::from(SDError::Internal(format!("service was null")));

        let b = Box::new(c_err);
        let ptr = Box::into_raw(b);

        unsafe {
            *error = ptr;
        }
        return -1;
    }

    let luser: &CStr = unsafe {
        CStr::from_ptr(user)
    };

    let u: String = match luser.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1;
        },
    };

    let lservice: &CStr = unsafe {
        CStr::from_ptr(service)
    };

    let s: String = match lservice.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1;
        },
    };

    let service = KeychainService::from(s);

    match get_keychain_item(&u, service) {
        Ok(sec) => {
            unsafe {
                *secret = CString::new(sec).expect("Failed to get keychain item secret").into_raw();
            }
            0
        },
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Set a keychain item for a user/service pair
///
/// Will return a failure code if the keychain item cannot be set
///
/// Parameters:
///
///     `user`: a pointer to a NULL-terminated string representing the user account name
///
///  `service`: a pointer to a NULL-terminated string representing the service subdomain
///
///   `secret`: a pointer to a NULL-terminated string representing the secret to store in the keychain
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
/// char * user = "user@safedrive.io";
/// char * service = "safedrive.io";
/// char * secret = "my_password";
/// SDDKError *error = NULL;
///
/// if (0 != sddk_set_keychain_item(user, service, secret, &error)) {
///     printf("Failed to set keychain item");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Set keychain item");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_set_keychain_item(user: *const std::os::raw::c_char,
                                         service: *const std::os::raw::c_char,
                                         secret: *const std::os::raw::c_char,
                                         mut error: *mut *mut SDDKError) -> std::os::raw::c_int {

    let luser: &CStr = unsafe {
        assert!(!user.is_null());
        CStr::from_ptr(user)
    };

    let u: String = match luser.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let lservice: &CStr = unsafe {
        assert!(!service.is_null());
        CStr::from_ptr(service)
    };

    let s: String = match lservice.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let lsecret: &CStr = unsafe {
        assert!(!secret.is_null());
        CStr::from_ptr(secret)
    };

    let sec: String = match lsecret.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let service = KeychainService::from(s);

    match set_keychain_item(&u, service, &sec) {
        Ok(()) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Delete a keychain item for a user/service pair
///
/// Will return a failure code if the keychain item cannot be deleted
///
/// Parameters:
///
///     `user`: a pointer to a NULL-terminated string representing the user account name
///
///  `service`: a pointer to a NULL-terminated string representing the service subdomain
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
/// char * user = "user@safedrive.io";
/// char * service = "safedrive.io";
/// SDDKError *error = NULL;
///
/// if (0 != sddk_delete_keychain_item(user, service, &error)) {
///     printf("Failed to delete keychain item");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Deleted keychain item");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_delete_keychain_item(user: *const std::os::raw::c_char,
                                            service: *const std::os::raw::c_char,
                                            mut error: *mut *mut SDDKError) -> std::os::raw::c_int {

    let luser: &CStr = unsafe {
        assert!(!user.is_null());
        CStr::from_ptr(user)
    };

    let u: String = match luser.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let lservice: &CStr = unsafe {
        assert!(!service.is_null());
        CStr::from_ptr(service)
    };

    let s: String = match lservice.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let service = KeychainService::from(s);

    match delete_keychain_item(&u, service) {
        Ok(()) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}


/// Login - must be called before any other function that interacts with the SFTP server
///
/// Will assert non-null on `state`, `username`, and `password`
///
///
/// Parameters:
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `username`: a NULL-terminated string representing a username for an account
///
///     `password`: a NULL-terminated string representing a password for an account
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
                             unique_client_id: *const std::os::raw::c_char,
                             username: *const std::os::raw::c_char,
                             password:  *const std::os::raw::c_char,
                             mut status: *mut *mut SDDKAccountStatus,
                             mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!state.is_null()); assert!(!username.is_null()); assert!(!password.is_null()); &mut * state };

    let uids: &CStr = unsafe {
        assert!(!unique_client_id.is_null());
        CStr::from_ptr(unique_client_id)
    };

    let uid: String = match uids.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let c_username: &CStr = unsafe { CStr::from_ptr(username) };
    let un: String =  match c_username.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let c_password: &CStr = unsafe { CStr::from_ptr(password) };
    let pa: String =  match c_password.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    match login(&uid, &un, &pa) {
        Ok((token, account_status)) => {
            {
                let ref s = account_status;

                c.0.set_account(Some(un.to_owned()), Some(pa.to_owned()));
                c.0.set_api_token(Some(token));
                c.0.set_unique_client_id(Some(uid));
                c.0.set_host(Some(s.host.clone()));
                c.0.set_port(Some(s.port));
                c.0.set_ssh_username(Some(s.userName.clone()));
            }
            let c_s = SDDKAccountStatus::from(account_status);

            let b = Box::new(c_s);
            let ptr = Box::into_raw(b);

            unsafe {
                *status = ptr;
            }
            0
        },
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}



/// Remove client
///
/// Will assert non-null on the `state` parameter only
///
///
/// Parameters:
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// if (0 != sddk_remove_client(&state, &error)) {
///     printf("Removing client failed");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Removing client successful");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_remove_client(state: *mut SDDKState,
                                     mut error: *mut *mut SDDKError) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!state.is_null()); &mut * state };

    match remove_software_client(c.0.get_api_token()) {
        Ok(()) => {},
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1;
        },
    };

    c.0.set_api_token(None);
    0
}





/// Load keys, must be called before any other function that starts a sync or restore
///
/// Will assert non-null on the `state` parameter only
///
///
/// Parameters:
///
///     `context`: an opaque pointer the caller can pass in and get access to in the callback
///
///     `context2`: an opaque pointer the caller can pass in and get access to in the callback
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `recovery_phrase`: a pointer to a recovery phrase obtained by previous calls. can be null if
///                        no phrase is available
///
///     `store_recovery_key`: a C function pointer that will be called when the app should store a
///                           recovery phrase
///
///     `store_recovery_key(new_phrase)`: a pointer to a recovery phrase that should be stored by the app.
///                                       note that the pointer becomes invalid after the callback function
///                                       returns
///
///     `issue`: a C function pointer that will be called to report non-fatal key loading issues
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
                                 context2: *mut std::os::raw::c_void,
                                 state: *mut SDDKState,
                                 mut error: *mut *mut SDDKError,
                                 recovery_phrase: *const std::os::raw::c_char,
                                 store_recovery_key: extern fn(store_context: *mut std::os::raw::c_void,
                                                               new_phrase: *mut std::os::raw::c_char),
                                 issue: extern fn(issue_context: *mut std::os::raw::c_void,
                                                  message: *mut std::os::raw::c_char)) -> std::os::raw::c_int {
    let mut c = unsafe{ assert!(!state.is_null()); &mut * state };

    let phrase: Option<String> = unsafe {
        if !recovery_phrase.is_null() {
            let c_recovery: &CStr = CStr::from_ptr(recovery_phrase);
            match c_recovery.to_str() {
                Ok(s) => Some(s.to_owned()),
                Err(err) => panic!("string is not valid UTF-8: {}", err),
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
    }, &|message| {
        /// call back to C to report non-fatal issue with keys
        let mut c_message = CString::new(message).unwrap();
        issue(context2, c_message.into_raw());
    }) {
        Ok(ks) => ks,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1;
        },
    };

    c.0.set_keys(Some(keyset.master), Some(keyset.main), Some(keyset.hmac), Some(keyset.tweak));
    0
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
///     `unique_client_id`: an uninitialized pointer that will be allocated and initialized when the function
///                         returns
///
///                         must be freed by the caller using `sddk_free_string()`
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

/// Get account status from the server
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
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `status`: an uninitialized pointer that will be allocated and initialized when the function
///               returns if the return value was 0
///
///               must be freed by the caller using `sddk_free_account_status()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1;
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


/// Get account details from the server
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
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `details`: an uninitialized pointer that will be allocated and initialized when the function
///                returns if the return value was 0
///
///                must be freed by the caller using `sddk_free_account_details()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1;
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

/// Get a list of all registered clients from the server
///
/// The caller does not own the memory pointed to by `clients` after this function returns, it must
/// be returned and freed by the library.
///
/// As a result, any data that the caller wishes to retain must be copied out of the buffer before
/// it is freed.
///
///
/// Parameters:
///
///     `username`: a NULL-terminated string representing a username for an account
///
///     `password`: a NULL-terminated string representing a password for an account
///
///     `clients`: an uninitialized pointer that will be allocated and initialized when the function
///                returns if the return value was 0
///
///                must be freed by the caller using `sddk_free_software_clients()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///     0+: number of registered clients found, and number of `SDDKSoftwareClient` structs allocated
///         in `client`
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
/// SDDKSoftwareClient * clients = NULL;
///
/// int length = sddk_get_clients(&state, &clients, &error);
///
/// if (0 != length) {
///     printf("Failed to get clients");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Got clients");
///     // do something with clients here, then free it
///     sddk_free_software_clients(&clients, length);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_get_software_clients(username: *const std::os::raw::c_char,
                                            password:  *const std::os::raw::c_char,
                                            mut clients: *mut *mut SDDKSoftwareClient,
                                            mut error: *mut *mut SDDKError) -> i64 {

    let c_username: &CStr = unsafe { CStr::from_ptr(username) };
    let un: String =  match c_username.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let c_password: &CStr = unsafe { CStr::from_ptr(password) };
    let pa: String =  match c_password.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let result = match get_software_clients(&un, &pa) {
        Ok(clients) => clients,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let b = Box::new(c_err);
            let ptr = Box::into_raw(b);

            unsafe {
                *error = ptr;
            }
            return -1;
        },
    };

    let cl = result.into_iter().map(|client| {
        SDDKSoftwareClient::from(client)
    }).collect::<Vec<SDDKSoftwareClient>>();;

    let mut b = cl.into_boxed_slice();
    let ptr = b.as_mut_ptr();
    let len = b.len();
    std::mem::forget(b);

    unsafe {
        *clients = ptr;
    }

    len as i64
}




/// Add a sync folder
///
/// Will return a failure code if for any reason the folder cannot be added
///
///
/// Parameters:
///
///        `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///         `name`: a NULL terminated string representing the folder name
///
///         `path`: a NULL terminated string representing the folder path in RFC3986 format
///
///    `encrypted`: an 8-bit integer representing a boolean where value >= 1 is true, 0 is false
///
///
///        `error`: an uninitialized pointer that will be allocated and initialized when the function
///                 returns if the return value was -1
///
///                 must be freed by the caller using `sddk_free_error()`
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
/// unsigned char encrypted = 1;
///
/// if (0 != sddk_add_sync_folder(&state, "Documents", "/Users/name/Documents", encrypted, &error)) {
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
                                       encrypted: std::os::raw::c_uchar,
                                       mut error: *mut *mut SDDKError) -> std::os::raw::c_longlong {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = match c_name.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };


    let c_path: &CStr = unsafe { CStr::from_ptr(path) };
    let p: String = match c_path.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let c_encrypted = encrypted >= 1;

    match add_sync_folder(c.0.get_api_token(), &n, &p, c_encrypted) {
        Ok(folder_id) => folder_id as i64,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Update a sync folder
///
/// Will return a failure code if for any reason the folder cannot be updated
///
///
/// Parameters:
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `name`: a NULL terminated string representing the folder name
///
///     `path`: a NULL terminated string representing the folder path in RFC3986 format
///
///     `syncing`: an 8-bit unsigned integer representing a boolean state: whether the folder
///                should be actively syncing at the current time or not. Valid settings are 0 or 1
///
///     `unique_id`: a 64-bit unsigned integer representing the unique folder ID
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
/// unsigned char syncing = 1;
///
/// if (0 != sddk_update_sync_folder(&state, "Documents", "/Users/name/Documents", syncing, 56, &error)) {
///     printf("Failed to update folder");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Updated folder");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_update_sync_folder(state: *mut SDDKState,
                                          name: *const std::os::raw::c_char,
                                          path: *const std::os::raw::c_char,
                                          syncing: std::os::raw::c_uchar,
                                          unique_id: std::os::raw::c_ulonglong,
                                          mut error: *mut *mut SDDKError) -> std::os::raw::c_longlong {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = match c_name.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };


    let c_path: &CStr = unsafe { CStr::from_ptr(path) };
    let p: String = match c_path.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    let c_syncing = syncing >= 1;

    let c_unique_id: u64 = unique_id;


    match update_sync_folder(c.0.get_api_token(), &n, &p, c_syncing, c_unique_id) {
        Ok(()) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

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
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `folder_id`: an unsigned 64-bit integer representing the registered folder ID
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Get a sync folder from the server
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
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `folder_id`: an unsigned 64-bit integer representing a registered folder ID
///
///     `folder`: an uninitialized pointer that will be allocated and initialized when the function
///               returns if the return value was 0
///
///               must be freed by the caller using `sddk_free_folder()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///     0+: number of registered folders found, and number of `SDDKFolder` structs allocated in folders
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
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            return -1;
        },
    };

    let f = SDDKFolder::from(nf);

    let boxed_folder = Box::new(f);
    let ptr = Box::into_raw(boxed_folder);

    unsafe {
        *folder = ptr;
    }

    0
}


/// Get a list of all sync folders from the server
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
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `folders`: an uninitialized pointer that will be allocated and initialized when the function
///                returns if the return value was 0
///
///                must be freed by the caller using `sddk_free_folders()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///     0+: number of registered folders found, and number of `SDDKFolder` structs allocated in folders
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
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            return -1;
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

/// Check a folder path for conflicts with existing registered sync folders
///
/// The caller does not own the memory pointed to by `error` after this function returns,
/// they must be returned and freed by the library.
///
/// As a result, any data that the caller wishes to retain must be copied out of the buffer before
/// it is freed.
///
///
/// Parameters:
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `folder_path`: a NULL terminated string representing the folder path to check
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
///
/// Return:
///
///     -1: failure, `error` will be set with more information
///
///     0: no conflicts
///
///     1: conflict found
///
/// # Examples
///
/// ```c
/// SDDKState *state; // retrieve from sddk_initialize()
/// SDDKError *error = NULL;
///
/// int res = sddk_has_conflicting_folder(&state, "/path/to/folder", &error)
///
/// if (res == 0) {
///     printf("No conflicts");
/// }
/// else if (res == 1) {
///     printf("Conflict found");
/// } else {
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_has_conflicting_folder(state: *mut SDDKState,
                                              folder_path: *const std::os::raw::c_char,
                                              mut error: *mut *mut SDDKError) -> i8 {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let c_name: &CStr = unsafe { CStr::from_ptr(folder_path) };
    let p: PathBuf = match c_name.to_str() {
        Ok(s) => PathBuf::from(s),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };


    match has_conflicting_folder(c.0.get_api_token(), &p) {
        Ok(conflict_found) => {
            if conflict_found {
                1
            } else {
                0
            }
        },
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}



/// Get a list of all sync sessions from the server
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
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `sessions`: an uninitialized pointer that will be allocated and initialized when the function
///                 returns if the return value was 0
///
///                 must be freed by the caller using `sddk_free_sessions()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
///
/// Return:
///
///      -1: failure, `error` will be set with more information
///
///      0+: number of sessions found, and number of `SDDKSyncSession` structs allocated in sessions
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
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            return -1;
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
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `session_id`: an unsigned 64-bit integer representing the session ID
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

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
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///  `schedule`: an `SDDKSyncCleaningSchedule` variant
///
///      `date`: the date string used for `SDDKSyncCleaningSchedule.ExactDateRFC*` variants. Should be
///              null for other enum variants.
///
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
                Err(err) => panic!("string is not valid UTF-8: {}", err),
            };

            Some(d)
        }
    };

    let sch = schedule.to_schedule(d);

    match clean_sync_sessions(c.0.get_api_token(), sch) {
        Ok(_) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

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
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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

/// Cancel a sync or restore by name
///
///
/// Parameters:
///
///     `name`: a NULL-terminated `UUIDv4` string representing the name of the sync session
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
/// SDDKError *error = NULL;
///
/// if (0 != sddk_cancel_sync_task("02c0dc9c-6217-407b-a3ef-0d7ac5f288b1", &error)) {
///     printf("Failed to cancel sync task");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Cancelled sync task");
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_cancel_sync_task(name: *const std::os::raw::c_char,
                                        mut error: *mut *mut SDDKError) -> std::os::raw::c_int {

    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = match c_name.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };

    cancel_sync_task(&n);

    0
}


/// Start a sync for the folder ID
///
///
/// Parameters:
///
///     `context`: an opaque pointer the caller can pass in and get access to in the progress callback
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `name`: a NULL-terminated `UUIDv4` string representing the name of the sync session
///
///     `folder_id`: an unsigned 64-bit integer representing a registered folder ID
///
///     `progress`: a C function pointer that will be called periodically to report progress
///
///     `issue`: a C function pointer that will be called periodically to report sync issues
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
                            context2: *mut std::os::raw::c_void,
                            state: *mut SDDKState,
                            mut error: *mut *mut SDDKError,
                            name: *const std::os::raw::c_char,
                            folder_id: std::os::raw::c_ulonglong,
                            progress: extern fn(context: *mut std::os::raw::c_void,
                                                context2: *mut std::os::raw::c_void,
                                                total: std::os::raw::c_ulonglong,
                                                current: std::os::raw::c_ulonglong,
                                                new_bytes: std::os::raw::c_ulonglong,
                                                percent: std::os::raw::c_double,
                                                tick: std::os::raw::c_uint),
                            issue: extern fn(context: *mut std::os::raw::c_void,
                                             context2: *mut std::os::raw::c_void,
                                             message: *const std::os::raw::c_char)) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };
    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = match c_name.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
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
               &mut |total, current, new_bytes, progress_percent, tick| {
                   let c_total: std::os::raw::c_ulonglong = total;
                   let c_current: std::os::raw::c_ulonglong = current;
                   let c_new_bytes: std::os::raw::c_ulonglong = new_bytes;
                   let c_percent: std::os::raw::c_double = progress_percent;
                   let c_tick: std::os::raw::c_uint =  if tick { 1 } else { 0 };
                   progress(context, context2, c_total, c_current, c_new_bytes, c_percent, c_tick);
               },
               &mut |message| {
                   let c_message = CString::new(message).expect("failed to get sync message");
                   issue(context, context2, c_message.as_ptr());
               }) {
        Ok(_) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Start a restore for the folder ID
///
/// Parameters:
///
///     `context`: an opaque pointer the caller can pass in and get access to in the progress callback
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `name`: a NULL-terminated UTF-8 string representing the `UUIDv4` name of the sync session
///
///     `folder_id`: an unsigned 64-bit integer representing a registered folder ID
///
///     `destination`: a NULL-terminated UTF-8 string representing the path the sync session will be restored to
///
///     `progress`: a C function pointer that will be called periodically to report progress
///
///     `issue`: a C function pointer that will be called periodically to report restore issues
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
                               context2: *mut std::os::raw::c_void,
                               state: *mut SDDKState,
                               mut error: *mut *mut SDDKError,
                               name: *const std::os::raw::c_char,
                               folder_id: std::os::raw::c_ulonglong,
                               destination: *const std::os::raw::c_char,
                               session_size: std::os::raw::c_ulonglong,
                               progress: extern fn(context: *mut std::os::raw::c_void,
                                                   context2: *mut std::os::raw::c_void,
                                                   total: std::os::raw::c_ulonglong,
                                                   current: std::os::raw::c_ulonglong,
                                                   new_bytes: std::os::raw::c_ulonglong,
                                                   percent: std::os::raw::c_double,
                                                   tick: std::os::raw::c_uint),
                               issue: extern fn(context: *mut std::os::raw::c_void,
                                                context2: *mut std::os::raw::c_void,
                                                message: *const std::os::raw::c_char)) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };
    let c_name: &CStr = unsafe { CStr::from_ptr(name) };
    let n: String = match c_name.to_str() {
        Ok(s) => s.to_owned(),
        Err(err) => panic!("string is not valid UTF-8: {}", err),
    };


    let c_destination: &CStr = unsafe { CStr::from_ptr(destination) };
    let d: String =  match c_destination.to_str() {
        Ok(p) => p.to_owned(),
        Err(err) => panic!("path is not valid UTF-8: {}", err),
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
                  &mut |total, current, new_bytes, progress_percent, tick| {
                      let c_total: std::os::raw::c_ulonglong = total;
                      let c_current: std::os::raw::c_ulonglong = current;
                      let c_percent: std::os::raw::c_double = progress_percent;
                      let c_new_bytes: std::os::raw::c_ulonglong = new_bytes;
                      let c_tick: std::os::raw::c_uint =  if tick { 1 } else { 0 };
                      progress(context, context2, c_total, c_current, c_new_bytes, c_percent, c_tick);
                  },
                  &mut |message| {
                      let c_message = CString::new(message).expect("failed to get sync message");
                      issue(context, context2, c_message.as_ptr());
                  }) {
        Ok(_) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}



/// Report an error to the server
///
/// Will return a failure code if the parameters are invalid or the request could not be completed
///
///
/// Parameters:
///
///     `unique_client_id`: a NULL terminated string representing the current UCID
///
///     `description`: a NULL terminated string representing the error description
///
///     `context`: a NULL terminated string representing any context that should be included in the
///              error report
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
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
                                    mut error: *mut *mut SDDKError) -> std::os::raw::c_int {

    let ucid: String = unsafe {
        assert!(!unique_client_id.is_null());

        let c_ucid: &CStr = CStr::from_ptr(unique_client_id);

        let ucid: String = match c_ucid.to_str() {
            Ok(s) => s.to_owned(),
            Err(err) => panic!("string is not valid UTF-8: {}", err),
        };

        ucid
    };

    let desc: String = unsafe {
        assert!(!description.is_null());

        let c_desc: &CStr = CStr::from_ptr(description);

        let desc: String = match c_desc.to_str() {
            Ok(s) => s.to_owned(),
            Err(err) => panic!("string is not valid UTF-8: {}", err),
        };

        desc
    };

    let cont: String = unsafe {
        assert!(!context.is_null());

        let c_cont: &CStr = CStr::from_ptr(context);

        let cont: String = match c_cont.to_str() {
            Ok(s) => s.to_owned(),
            Err(err) => panic!("string is not valid UTF-8: {}", err),
        };

        cont
    };

    let ver: Option<String> = unsafe {
        if client_version.is_null() {
            None
        } else {
            let c_client_version: &CStr = CStr::from_ptr(client_version);

            let ver: String = match c_client_version.to_str() {
                Ok(s) => s.to_owned(),
                Err(err) => panic!("string is not valid UTF-8: {}", err),
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
                Err(err) => panic!("string is not valid UTF-8: {}", err),
            };

            Some(os)
        }
    };


    match send_error_report(ver, os, &ucid, &desc, &cont) {
        Ok(()) => {
            0
        },
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Log
///
/// Will write to stdout if a terminal is present, otherwise will log to a file in the storage dir
/// as well as an in-memory ring buffer that will be sent with any error reports
///
///
/// Parameters:
///
///     `message`: a NULL terminated string for the log message
///
///     `level`: an SDDKLogLevel
///
///
///
/// # Examples
///
/// ```c
/// sddk_log("starting safedrive", SDDKLogLevelInfo);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_log(message: *const std::os::raw::c_char,
                           level: SDDKLogLevel) {

    let msg: String = unsafe {
        assert!(!message.is_null());

        let c_msg: &CStr = CStr::from_ptr(message);

        let msg: String = match c_msg.to_str() {
            Ok(s) => s.to_owned(),
            Err(err) => panic!("string is not valid UTF-8: {}", err),
        };

        msg
    };

    let log_level = match level {
        SDDKLogLevel::Error => ::log::LogLevelFilter::Error,
        SDDKLogLevel::Warn => ::log::LogLevelFilter::Warn,
        SDDKLogLevel::Info => ::log::LogLevelFilter::Info,
        SDDKLogLevel::Debug => ::log::LogLevelFilter::Debug,
        SDDKLogLevel::Trace => ::log::LogLevelFilter::Trace,
    };

    log(&msg, log_level);
}




/// Free a pointer to a list of software clients
///
/// Note: This is *not* the same as calling `free()` in C, they are not interchangeable
///
/// Parameters:
///
///     `clients`: a pointer obtained from calling `sddk_get_software_clients()`
///
///     `length`: number of `SDDKSoftwareClient` structs that were allocated in the pointer
///
///
/// # Examples
///
/// ```c
///sddk_free_software_clients(&clients, length);
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_free_software_clients(clients: *mut *mut SDDKSoftwareClient,
                                             length: u64) {
    assert!(!clients.is_null());
    let l = length as usize;

    let clients: Vec<SDDKSoftwareClient> = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(*clients, l)).into_vec() };
    for client in clients {
        let _ = unsafe { CString::from_raw(client.unique_client_id as *mut std::os::raw::c_char) };
        let _ = unsafe { CString::from_raw(client.language as *mut std::os::raw::c_char) };
        let _ = unsafe { CString::from_raw(client.operating_system as *mut std::os::raw::c_char) };
    }
}


/// Free a pointer to a sync folder
///
/// Note: This is *not* the same as calling `free()` in C, they are not interchangeable
///
/// Parameters:
///
///     `folder`: a pointer obtained from calling `sddk_get_sync_folder()`
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
/// Note: This is *not* the same as calling `free()` in C, they are not interchangeable
///
/// Parameters:
///
///     `folders`: a pointer obtained from calling `sddk_get_sync_folders()`
///
///     `length`: number of `SDDKFolder` structs that were allocated in the pointer
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
/// Note: This is *not* the same as calling `free()` in C, they are not interchangeable
///
/// Parameters:
///
///     `sessions`: a pointer obtained from calling `sddk_get_sync_sessions()`
///
///     `length`: number of `SDDKSyncSession` structs that were allocated in the pointer
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

/// Free an opaque pointer to an `SDDKState`
///
/// Note: This is *not* the same as calling `free()` in C, they are not interchangeable
///
/// Parameters:
///
///     `state`: a pointer obtained from calling `sddk_initialize()`
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

    let _: Box<SDDKState> = unsafe { Box::from_raw(*state) };
}

/// Free a pointer to a string
///
/// Note: This is *not* the same as calling `free()` in C, they are not interchangeable
///
///
/// Parameters:
///
///     `string`: a pointer obtained from calling a function that returns a C string
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



/// Free a pointer to an `SDDKError`
///
/// Note: This is *not* the same as calling `free()` in C, they are not interchangeable
///
/// Parameters:
///
///     `error`: a pointer to an `SDDKError` obtained from another call
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

/// Free a pointer to an `SDDKAccountStatus`
///
/// Note: This is *not* the same as calling `free()` in C, they are not interchangeable
///
/// Parameters:
///
///     `status`: a pointer to an `SDDKStatus` obtained from another call
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
    if !s.time.is_null() {
        let _ = unsafe {
            Box::from_raw(s.time as *mut std::os::raw::c_ulonglong)
        };
    }
}

/// Free a pointer to an `SDDKAccountDetails`
///
/// Note: This is *not* the same as calling `free()` in C, they are not interchangeable
///
/// Parameters:
///
///     `details`: a pointer to an `SDDKAccountDetails` obtained from another call
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
    /*if !d.notifications.is_null() {
        let _ = unsafe {
            let notifications: Vec<SDDKNotification> = Box::from_raw(std::slice::from_raw_parts_mut(d.notifications as *mut SDDKNotification, d.notification_count as usize)).into_vec();
            for notification in notifications {
                let _ = CString::from_raw(notification.title as *mut std::os::raw::c_char);
                let _ = CString::from_raw(notification.message as *mut std::os::raw::c_char);

            }
        };
    }*/
}


/// Create a directory on the remote filesystem
///
/// Parameters:
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
///
///     `remote_path`: a NULL terminated string representing the path of the directory to create
///
///
/// # Examples
///
/// ```c
/// if (0 != sddk_remote_mkdir(state, &error, "/storage/test") {
///     printf("Failed to create directory");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Directory created");
///
/// }
/// ```
///
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_remote_mkdir(state: *mut SDDKState, mut error: *mut *mut SDDKError, remote_path: *const std::os::raw::c_char) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let c_path: String = unsafe {
        assert!(!remote_path.is_null());

        let c_path: &CStr = CStr::from_ptr(remote_path);

        let path: String = match c_path.to_str() {
            Ok(s) => s.to_owned(),
            Err(err) => panic!("string is not valid UTF-8: {}", err),
        };

        path
    };

    let p = PathBuf::from(c_path);

    match remote_mkdir(c.0.get_host(), c.0.get_port(), &p, c.0.get_ssh_username(), c.0.get_account_password()) {
        Ok(()) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Remove a directory on the remote filesystem
///
/// Parameters:
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
///
///     `remote_path`: a NULL terminated string representing the path of the directory to remove
///
///
/// # Examples
///
/// ```c
/// if (0 != sddk_remote_rmdir(state, &error, "/storage/test") {
///     printf("Failed to remove directory");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Directory removed");
///
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_remote_rmdir(state: *mut SDDKState, mut error: *mut *mut SDDKError, remote_path: *const std::os::raw::c_char) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let c_path: String = unsafe {
        assert!(!remote_path.is_null());

        let c_path: &CStr = CStr::from_ptr(remote_path);

        let path: String = match c_path.to_str() {
            Ok(s) => s.to_owned(),
            Err(err) => panic!("string is not valid UTF-8: {}", err),
        };

        path
    };

    let p = PathBuf::from(c_path);

    match remote_rmdir(c.0.get_host(), c.0.get_port(), &p, c.0.get_ssh_username(), c.0.get_account_password()) {
        Ok(()) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Remove a path on the remote filesystem
///
/// Parameters:
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
///
///     `remote_path`: a NULL terminated string representing the path of the directory to remove
///
///     `recursive`: an 8-bit integer representing a boolean where value >= 1 is true, 0 is false
///
///
/// # Examples
///
/// ```c
/// if (0 != sddk_remote_rm(state, &error, "/storage/test", 1) {
///     printf("Failed to remove path");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Path removed");
///
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_remote_rm(state: *mut SDDKState, mut error: *mut *mut SDDKError, remote_path: *const std::os::raw::c_char, recursive: std::os::raw::c_uchar) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let c_path: String = unsafe {
        assert!(!remote_path.is_null());

        let c_path: &CStr = CStr::from_ptr(remote_path);

        let path: String = match c_path.to_str() {
            Ok(s) => s.to_owned(),
            Err(err) => panic!("string is not valid UTF-8: {}", err),
        };

        path
    };

    let p = PathBuf::from(c_path);

    let c_recursive = recursive >= 1;


    match remote_rm(c.0.get_host(), c.0.get_port(), &p, c_recursive, c.0.get_ssh_username(), c.0.get_account_password()) {
        Ok(()) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}

/// Move a directory on the remote filesystem
///
/// Parameters:
///
///     `state`: an opaque pointer obtained from calling `sddk_initialize()`
///
///     `error`: an uninitialized pointer that will be allocated and initialized when the function
///              returns if the return value was -1
///
///              must be freed by the caller using `sddk_free_error()`
///
///     `remote_path`: a NULL terminated string representing the path of the file to move
///
///     `new_path`: a NULL terminated string representing the new path of the file to move
///
/// # Examples
///
/// ```c
/// if (0 != sddk_remote_mv(state, &error, "/storage/test", "/storage/test2") {
///     printf("Failed to move path");
///     // do something with error here, then free it
///     sddk_free_error(&error);
/// }
/// else {
///     printf("Path moved");
///
/// }
/// ```
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn sddk_remote_mv(state: *mut SDDKState, mut error: *mut *mut SDDKError, remote_path: *const std::os::raw::c_char, new_path: *const std::os::raw::c_char) -> std::os::raw::c_int {
    let c = unsafe{ assert!(!state.is_null()); &mut * state };

    let c_path: String = unsafe {
        assert!(!remote_path.is_null());

        let c_path: &CStr = CStr::from_ptr(remote_path);

        let path: String = match c_path.to_str() {
            Ok(s) => s.to_owned(),
            Err(err) => panic!("string is not valid UTF-8: {}", err),
        };

        path
    };

    let c_newpath: String = unsafe {
        assert!(!new_path.is_null());

        let c_newpath: &CStr = CStr::from_ptr(new_path);

        let path: String = match c_newpath.to_str() {
            Ok(s) => s.to_owned(),
            Err(err) => panic!("string is not valid UTF-8: {}", err),
        };

        path
    };

    let path = PathBuf::from(c_path);

    let new_path = PathBuf::from(c_newpath);

    match remote_mv(c.0.get_host(), c.0.get_port(), &path, &new_path, c.0.get_ssh_username(), c.0.get_account_password()) {
        Ok(()) => 0,
        Err(err) => {
            let c_err = SDDKError::from(err);

            let boxed_error = Box::new(c_err);
            let ptr = Box::into_raw(boxed_error);

            unsafe {
                *error = ptr;
            }
            -1
        },
    }
}
