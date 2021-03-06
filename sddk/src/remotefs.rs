
use ssh2::{Session, Sftp, Channel};
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

    pub fn mv(&mut self, remote_path: &Path, new_path: &Path) -> Result<(), SDError> {
        self.connect()?;

        debug!("mv");

        let ses: &Session = match self.session {
            Some(ref ses) => ses,
            None => return Err(SDError::Internal("no ssh session available".to_string()))
        };

        let sftp: Sftp = match ses.sftp() {
            Ok(sftp) => sftp,
            Err(_) => return Err(SDError::Internal("no sftp channel available".to_string()))
        };

        debug!("mv: {} -> {}", remote_path.display(), new_path.display());

        sftp.rename(remote_path, new_path, None)?;

        Ok(())
    }

    pub fn mkdir(&mut self, recursive: bool, remote_path: &Path) -> Result<(), SDError> {
        self.connect()?;

        let ses: &Session = match self.session {
            Some(ref ses) => ses,
            None => return Err(SDError::Internal("no ssh session available".to_string()))
        };

        let sftp: Sftp = match ses.sftp() {
            Ok(sftp) => sftp,
            Err(_) => return Err(SDError::Internal("no sftp channel available".to_string()))
        };

        debug!("mkdir: {}", remote_path.display());

        if recursive {
            debug!("mkdir: mode recursive");

            let mut current: PathBuf = PathBuf::from("/");

            for component in remote_path.components() {
                debug!("mkdir: directory component check: {:?}", component);

                match component {
                    Component::Prefix(_) => {

                    },
                    Component::RootDir => {

                    },
                    Component::CurDir => {

                    },
                    Component::ParentDir => {

                    },
                    Component::Normal(s) =>{
                        let mut path = PathBuf::new();
                        path.push(s);
                        current.push(&path);
                        match sftp.opendir(&current) {
                            Ok(_) => {
                                debug!("mkdir: directory component already exists {:?}", component);
                            },
                            Err(_) => {
                                sftp.mkdir(&current, 0o755)?;
                            }
                        }
                    },
                }
            }
        } else {
            debug!("mkdir: mode normal");

            match sftp.opendir(remote_path) {
                Ok(_) => {
                    debug!("mkdir: directory already exists {}", remote_path.display());
                },
                Err(_) => {
                    sftp.mkdir(remote_path, 0o755)?;
                }
            }

        }

        Ok(())
    }

    pub fn rmdir(&mut self, remote_path: &Path) -> Result<(), SDError> {
        self.connect()?;

        debug!("rmdir");

        let ses: &Session = match self.session {
            Some(ref ses) => ses,
            None => return Err(SDError::Internal("no ssh session available".to_string()))
        };

        let sftp: Sftp = match ses.sftp() {
            Ok(sftp) => sftp,
            Err(_) => return Err(SDError::Internal("no sftp channel available".to_string()))
        };

        debug!("rmdir: {}", remote_path.display());

        sftp.rmdir(remote_path)?;

        Ok(())
    }

    pub fn rm(&mut self, remote_path: &Path, recursive: bool) -> Result<(), SDError> {
        self.connect()?;

        debug!("rm");

        let ses: &Session = match self.session {
            Some(ref ses) => ses,
            None => return Err(SDError::Internal("no ssh session available".to_string()))
        };

        // we use remote commands for recursive because SFTP has no "rm -r" equivalent in `unlink()`
        if recursive {
            let mut channel: Channel = match ses.channel_session() {
                Ok(sftp) => sftp,
                Err(_) => return Err(SDError::Internal("no exec channel available".to_string()))
            };

            debug!("rm -r: {}", remote_path.display());

            match remote_path.to_str() {
                Some(s) => {
                    let command = format!("rm -rf {}", s);

                    channel.exec(&command)?;

                    Ok(())
                },
                None => Err(SDError::UnicodeError)
            }
        } else {
            debug!("rm: {}", remote_path.display());

            match ses.sftp() {
                Ok(sftp) => {

                    sftp.unlink(remote_path)?;

                    Ok(())
                },
                Err(_) => Err(SDError::Internal("no sftp channel available".to_string()))
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "network_tests")]
mod remote_fs_test {
    use super::*;
    use rand;
    use rand::Rng;

    static USERNAME: &str = env!("TEST_USER");
    static PASSWORD: &str = env!("TEST_PASSWORD");
    
    static HOST: &str = "sftp-client.safedrive.io";
    static PORT: u16 = 2221;

    #[test]
    fn connect() {
        let mut remote_fs = RemoteFS::new(HOST, PORT, USERNAME, PASSWORD);

        match remote_fs.connect() {
            Ok(()) => {

            },
            Err(err) => {
                panic!("{}", err);
            },
        };
    }

    #[test]
    fn mkdir_flat() {
        let mut remote_fs = RemoteFS::new(HOST, PORT, USERNAME, PASSWORD);
        let s = rand::thread_rng()
            .gen_ascii_chars()
            .take(10)
            .collect::<String>();
        let root = PathBuf::from("/storage");

        let path = root.join(format!("test-{}", s));

        match remote_fs.mkdir(false, &path) {
            Ok(()) => {

            },
            Err(err) => {
                panic!("{}", err);
            },
        };
    }

    #[test]
    fn mkdir_recursive() {
        let s = rand::thread_rng()
            .gen_ascii_chars()
            .take(10)
            .collect::<String>();

        let mut remote_fs = RemoteFS::new(HOST, PORT, USERNAME, PASSWORD);

        let root = PathBuf::from("/storage");

        let path = root.join(format!("test-{}/{}", s, s));

        match remote_fs.mkdir(true, &path) {
            Ok(()) => {

            },
            Err(err) => {
                panic!("{}", err);
            },
        };
    }


    #[test]
    #[should_panic]
    fn mkdir_nonrecursive_not_exists() {
        let s = rand::thread_rng()
            .gen_ascii_chars()
            .take(10)
            .collect::<String>();

        let mut remote_fs = RemoteFS::new(HOST, PORT, USERNAME, PASSWORD);

        let root = PathBuf::from("/storage");

        let path = root.join(format!("test-{}/{}", s, s));

        match remote_fs.mkdir(false, &path) {
            Ok(()) => {

            },
            Err(err) => {
                panic!("{}", err);
            },
        };
    }

    #[test]
    fn mv_flat() {
        let mut remote_fs = RemoteFS::new(HOST, PORT, USERNAME, PASSWORD);
        let s = rand::thread_rng()
            .gen_ascii_chars()
            .take(10)
            .collect::<String>();
        let root = PathBuf::from("/storage");

        let path = root.join(format!("test-{}", s));

        match remote_fs.mkdir(false, &path) {
            Ok(()) => {

            },
            Err(err) => {
                panic!("{}", err);
            },
        };

        let new_path = root.join(format!("testmoved-{}", s));


        match remote_fs.mv(&path, &new_path) {
            Ok(()) => {

            },
            Err(err) => {
                panic!("{}", err);
            },
        };
    }

    #[test]
    fn rmdir_flat() {
        let mut remote_fs = RemoteFS::new(HOST, PORT, USERNAME, PASSWORD);
        let s = rand::thread_rng()
            .gen_ascii_chars()
            .take(10)
            .collect::<String>();
        let root = PathBuf::from("/storage");

        let path = root.join(format!("test-{}", s));

        match remote_fs.mkdir(false, &path) {
            Ok(()) => {

            },
            Err(err) => {
                panic!("{}", err);
            },
        };

        match remote_fs.rmdir(&path) {
            Ok(()) => {

            },
            Err(err) => {
                panic!("{}", err);
            },
        };
    }
}

