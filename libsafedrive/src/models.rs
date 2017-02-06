#![allow(non_snake_case)]

use std;

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum SyncVersion {
    Version0, // doesn't exist
    Version1, // testing format
    Version2, // production
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
            SyncVersion::Version0 => {
                write!(f, "0")
            },
            SyncVersion::Version1 => {
                write!(f, "1")
            },
            SyncVersion::Version2 => {
                write!(f, "2")
            },
        }
    }
}

// responses

//private final String token
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    pub token: String,
}

//private final int id
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFolderResponse {
    pub id: u64,
}

//private final String status;
//private final String host;
//private final int port;
//private final String userName;
//private final Long time;
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountStatus {
    pub status: Option<String>,
    pub host: String,
    pub port: u16,
    pub userName: String,
    pub time: Option<u64>
}



//private final long assignedStorage;
//private final long usedStorage;
//private final int lowFreeStorageThreshold;
//private final long expirationDate;
//private final Set<NotificationTO> notifications;
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountDetails {
    pub assignedStorage: u64,
    pub usedStorage: u64,
    pub lowFreeStorageThreshold: i64,
    pub expirationDate: u64,
    pub notifications: Option<Vec<Notification>>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    pub title: String,
    pub message: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WrappedKeysetBody {
    pub master: String,
    pub main: String,
    pub hmac: String,
    pub tweak: String
}

pub struct SyncSessionResponse<'a> {
    pub name: &'a str,
    pub folder_id: u64,
    pub chunk_data: Vec<u8>
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
    pub encrypted: bool
}

