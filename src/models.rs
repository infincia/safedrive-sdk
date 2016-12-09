extern crate libc;
use std::path::{PathBuf};

extern crate time;
use self::time::Timespec;

// c models

#[derive(Debug)]
pub struct Context {
    pub db_path: PathBuf,
    pub unique_client_path: PathBuf,
    pub storage_path: PathBuf,
    pub unique_client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_token: Option<String>,
    pub ssh_username: Option<String>,
    pub ssh_password: Option<String>,
    pub safedrive_sftp_client_ip: Option<String>,
    pub safedrive_sftp_client_port: Option<u16>,
    pub main_key: Option<[u8; 32]>,
    pub hmac_key: Option<[u8; 32]>,
}

impl Context {
    pub fn new(db_path: PathBuf, unique_client_path: PathBuf, storage_path: PathBuf, unique_client_id: String) -> Context {
        Context {
            db_path: db_path,
            unique_client_path: unique_client_path,
            storage_path: storage_path,
            unique_client_id: unique_client_id,
            username: None,
            password: None,
            api_token: None,
            ssh_username: None,
            ssh_password: None,
            safedrive_sftp_client_ip: None,
            safedrive_sftp_client_port: None,
            main_key: None,
            hmac_key: None
        }
    }

    pub fn set_keys(&mut self, main_key: [u8; 32], hmac_key: [u8; 32]) {
        self.main_key = Some(main_key);
        self.hmac_key = Some(hmac_key);
    }

    pub fn get_api_token(&self) -> &String {
        let tok = match self.api_token {
            Some(ref t) => t,
            None => panic!("attempt to use api token before setting")
        };

        tok
    }

    pub fn set_api_token(&mut self, token: String) {
        self.api_token = Some(token);
    }


    pub fn get_unique_client_id(&self) -> &String {
        &self.unique_client_id
    }

    pub fn set_account(&mut self, username: String, password: String) {
        self.username = Some(username);
        self.password = Some(password);
    }

    pub fn get_account_username(&self) -> &String {
        let username = match self.username {
            Some(ref u) => u,
            None => panic!("attempt to use account username before setting")
        };

        username
    }

    pub fn get_account_password(&self) ->  &String {
        let password = match self.password {
            Some(ref p) => p,
            None => panic!("attempt to use account password before setting")
        };
        password

    }

    pub fn set_ssh_credentials(&mut self, username: String, password: String, ip: String, port: u16) {
        self.ssh_username = Some(username);
        self.ssh_password = Some(password);
        self.safedrive_sftp_client_ip = Some(ip);
        self.safedrive_sftp_client_port = Some(port);
    }

    pub fn get_ssh_username(&self) -> &String {
        let username = match self.ssh_username {
            Some(ref u) => u,
            None => panic!("attempt to use ssh username before setting")
        };

        username
    }

    pub fn get_ssh_password(&self) ->  &String {
        let password = match self.ssh_password {
            Some(ref p) => p,
            None => panic!("attempt to use ssh password before setting")
        };
        password

    }

    pub fn get_safedrive_sftp_client_ip(&self) -> &String {
        let ip = match self.safedrive_sftp_client_ip {
            Some(ref i) => i,
            None => panic!("attempt to use ip before setting")
        };
        ip
    }
    pub fn get_safedrive_sftp_client_port(&self) -> u16 {
        self.safedrive_sftp_client_port.expect("attenpt to use port before setting")
    }

    pub fn get_main_key(&self) -> [u8; 32] {
        self.main_key.expect("Attempted to use main key before it was set")
    }

    pub fn get_hmac_key(&self) -> [u8; 32] {
        self.hmac_key.expect("Attempted to use hmac key before it was set")
    }
}

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