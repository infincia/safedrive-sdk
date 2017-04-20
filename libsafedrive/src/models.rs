#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std;

#[derive(Debug)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum SyncSchedule {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug)]
pub enum SyncCleaningSchedule {
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
    ExactDateRFC3339 { date: String },
    ExactDateRFC2822 { date: String },

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

impl std::fmt::Display for SyncCleaningSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            SyncCleaningSchedule::Auto => {
                write!(f, "automatic")
            },
            SyncCleaningSchedule::ExactDateRFC3339 { .. } => {
                write!(f, "from RFC3339 string")
            },
            SyncCleaningSchedule::ExactDateRFC2822 { .. } => {
                write!(f, "from RFC 2822 string")
            },
            SyncCleaningSchedule::All => {
                write!(f, "all")
            },
            SyncCleaningSchedule::BeforeToday => {
                write!(f, "before today")
            },
            SyncCleaningSchedule::BeforeThisWeek => {
                write!(f, "before this week")
            },
            SyncCleaningSchedule::BeforeThisMonth => {
                write!(f, "before this month")
            },
            SyncCleaningSchedule::BeforeThisYear => {
                write!(f, "before this year")
            },
            SyncCleaningSchedule::OneDay => {
                write!(f, "older than 24 hours")
            },
            SyncCleaningSchedule::OneWeek => {
                write!(f, "older than 7 days")
            },
            SyncCleaningSchedule::OneMonth => {
                write!(f, "older than 30 days")
            },
            SyncCleaningSchedule::OneYear => {
                write!(f, "older than 365 days")
            },
        }
    }
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum SyncVersion {
    Version0, // doesn't exist
    Version1, // testing format
    Version2, // production
}

impl SyncVersion {
    pub fn leading_value_size(&self) -> usize {
        match *self {
            SyncVersion::Version0 => panic!("invalid version"),
            SyncVersion::Version1 => 18 as usize,
            SyncVersion::Version2 => 18 as usize,
        }
    }

    pub fn window_size_bits(&self) -> u32 {
        match *self {
            SyncVersion::Version0 => panic!("invalid version"),
            SyncVersion::Version1 => 6,
            SyncVersion::Version2 => 6,
        }
    }

    pub fn compression(&self) -> CompressionType {
        match *self {
            SyncVersion::Version0 => panic!("invalid version"),
            SyncVersion::Version1 => CompressionType::None,
            SyncVersion::Version2 => CompressionType::Lz4,
        }
    }

    pub fn min_chunk_size(&self) -> usize {
        match *self {
            SyncVersion::Version0 => panic!("invalid version"),
            SyncVersion::Version1 => 0,
            SyncVersion::Version2 => 2048,
        }
    }

    pub fn max_chunk_size(&self) -> usize {
        match *self {
            SyncVersion::Version0 => panic!("invalid version"),
            SyncVersion::Version1 => ::std::usize::MAX,
            SyncVersion::Version2 => 1_000_000,
        }
    }
}

impl std::default::Default for SyncVersion {
    fn default() -> SyncVersion {
        SyncVersion::Version1
    }
}

impl AsRef<[u8]> for SyncVersion {
    fn as_ref(&self) -> &[u8] {
        match *self {
            SyncVersion::Version0 => "00".as_bytes(),
            SyncVersion::Version1 => "01".as_bytes(),
            SyncVersion::Version2 => "02".as_bytes(),
        }
    }
}

impl std::fmt::Display for SyncVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            SyncVersion::Version0 => write!(f, "0"),
            SyncVersion::Version1 => write!(f, "1"),
            SyncVersion::Version2 => write!(f, "2"),
        }
    }
}

/// binary flags


bitflags! {
    pub flags BinaryFlags: u8 {
        const Empty      = 0b00000000,
        const Production = 0b00000001,
        const Stable     = 0b00000010,
        const Beta       = 0b00000100,
        const Nightly    = 0b00001000,
        const Compressed = 0b00010000,
    }
}

/// responses

///private final String token
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    pub token: String,
}

///private final String uniqueId
///private final String operatingSystem
///private final String language
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SoftwareClient {
    pub uniqueId: String,
    pub operatingSystem: String,
    pub language: String,
}

impl std::fmt::Display for SoftwareClient {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "SoftwareClient {{ uniqueId: {}, os: {}, language: {} }}", self.uniqueId, self.operatingSystem, self.language)
    }
}


///private final int id
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFolderResponse {
    pub id: u64,
}

///private final String status;
///private final String host;
///private final int port;
///private final String userName;
///private final Long time;
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountStatus {
    pub status: Option<String>,
    pub host: String,
    pub port: u16,
    pub userName: String,
    pub time: Option<u64>,
}



///private final long assignedStorage;
///private final long usedStorage;
///private final int lowFreeStorageThreshold;
///private final long expirationDate;
///private final Set<NotificationTO> notifications;
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountDetails {
    pub assignedStorage: u64,
    pub usedStorage: u64,
    pub lowFreeStorageThreshold: i64,
    pub expirationDate: u64,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    pub title: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WrappedKeysetBody {
    pub master: String,
    pub main: String,
    pub hmac: String,
    pub tweak: String,
}

pub struct SyncSessionResponse<'a> {
    pub name: &'a str,
    pub folder_id: u64,
    pub chunk_data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerErrorResponse {
    pub message: String,
}


/*
Current sync folder model:

"id" : 1,
"folderName" : "Music",
"folderPath" : /Volumes/MacOS/Music,
"addedDate"  : 1435864769463,
"encrypted"  : false
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisteredFolder {
    pub id: u64,
    pub folderName: String,
    pub folderPath: String,
    pub addedDate: u64,
    pub encrypted: bool,
    pub syncing: bool,
}
