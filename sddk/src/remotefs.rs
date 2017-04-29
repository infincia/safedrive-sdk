#![cfg(target_os = "macos")]

use ssh2::{Session, Sftp};
use std::net::TcpStream;
use std::path::{Path, PathBuf, Component};


use error::SDError;


pub struct RemoteFS<'a> {
    host: &'a str,
    port: u16,
    username: &'a str,
    password: &'a str,
    tcp_stream: Option<TcpStream>,
    session: Option<Session>,
}

impl<'a> RemoteFS<'a> {
    pub fn new(host: &'a str, port: u16, username: &'a str, password: &'a str) -> RemoteFS<'a> {
        RemoteFS {
            host: host,
            port: port,
            username: username,
            password: password,
            tcp_stream: None,
            session: None,
        }
    }
}
