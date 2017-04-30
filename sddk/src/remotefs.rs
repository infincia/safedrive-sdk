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

    pub fn connect(&mut self) -> Result<(), SDError> {
        match self.session {
            Some(_) => {
                debug!("reusing ssh session");
                Ok(())
            },
            None => {
                debug!("creating new ssh session");

                debug!("connecting to url {}:{}", self.host, self.port);

                let tcp = TcpStream::connect((self.host, self.port))?;

                debug!("opening session on {:?}", tcp);

                let mut session: Session = match Session::new() {
                    Some(session) => session,
                    None => return Err(SDError::Internal("no session available".to_string()))
                };

                debug!("running ssh handshake");

                session.handshake(&tcp)?;

                debug!("authenticating");

                session.userauth_password(self.username, self.password)?;

                if session.authenticated() {
                    debug!("session authenticated");
                    self.session = Some(session);
                    self.tcp_stream = Some(tcp);
                    Ok(())
                } else {
                    debug!("session authentication failed");

                    self.session = None;
                    self.tcp_stream = None;
                    Err(SDError::Authentication)
                }
            }
        }
    }



}

