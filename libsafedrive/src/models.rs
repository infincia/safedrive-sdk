extern crate libc;

extern crate time;
use self::time::Timespec;

#[derive(Debug)]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub path: String,
}

#[derive(Debug)]
pub struct SyncSession {
    pub id: i64,
    pub filename: String,
    pub size: i64,
    pub date: Timespec,
    pub folder_id: i32,
}

#[derive(Debug)]
pub struct Block {
    pub id: i64,
    pub synced: i32,
    pub refcount: i32,
    pub hmac: Option<Vec<u8>>,
}

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

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientRegisterRequest {
    pub operatingSystem: String,
    pub email: String,
    pub password: String,
    pub language: String,
    pub uniqueClientId: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct WrappedKeyset {
    pub master: String,
    pub main: String,
    pub hmac: String
}

//private final String status;
//private final String host;
//private final int port;
//private final String userName;
//private final Long time;
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
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

