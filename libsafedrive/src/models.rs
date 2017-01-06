#![allow(non_snake_case)]

extern crate libc;

#[derive(Debug)]
pub enum Configuration {
    Staging,
    Production
}

// request bodies

#[derive(Serialize, Debug)]
pub struct RegisterClient<'a> {
    pub operatingSystem: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub language: &'a str,
    pub uniqueClientId: &'a str
}

#[derive(Serialize, Debug)]
pub struct AccountKey<'a> {
    pub master: &'a str,
    pub main: &'a str,
    pub hmac: &'a str
}

#[derive(Serialize, Debug)]
pub struct CreateFolder<'a> {
    pub folderName: &'a str,
    pub folderPath: &'a str,
    pub encrypted: bool
}

// responses

//private final String token
#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub token: String,
}

//private final String uniqueClientId
#[derive(Serialize, Deserialize, Debug)]
pub struct UniqueClientID {
    pub id: String,
}

#[derive(Debug)]
pub struct Block<'a> {
    pub chunk_data: Vec<u8>,
    pub name: &'a str
}

//private final int id
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFolderResponse {
    pub id: i32,
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
pub struct WrappedKeyset {
    pub master: String,
    pub main: String,
    pub hmac: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncSession {
    pub folder_id: i32,
    pub name: String,
    pub size: u64,
    pub time: u64,
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

