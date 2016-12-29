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