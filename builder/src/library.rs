
#[derive(Debug, Clone, Copy)]
pub enum Library {
    Libsodium,
    Libressl,
    Libssh2
}

impl AsRef<str> for Library {
    fn as_ref(&self) -> &str {
        match *self {
            Library::Libsodium => {
                "libsodium"
            },
            Library::Libressl => {
                "libressl"
            },
            Library::Libssh2 => {
                "libssh2"
            }
        }
    }
}

impl Library {
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    pub fn version(&self) -> &str {
        match *self {
            Library::Libsodium => {
                "1.0.12"
            },
            Library::Libressl => {
                "2.5.4"
            },
            Library::Libssh2 => {
                "1.8.0"
            }
        }
    }

    pub fn version_file(&self) -> &str {
        match *self {
            Library::Libsodium => {
                "sodium_ver"
            },
            Library::Libressl => {
                "libressl_ver"
            },
            Library::Libssh2 => {
                "ssh2_ver"
            }
        }
    }

    pub fn needs_cmake(&self) -> bool {
        match *self {
            Library::Libsodium => {
                false
            },
            Library::Libressl => {
                true
            },
            Library::Libssh2 => {
                true
            }
        }
    }

    pub fn needs_msbuild(&self) -> bool {
        match *self {
            Library::Libsodium => {
                true
            },
            Library::Libressl => {
                true
            },
            Library::Libssh2 => {
                true
            }
        }
    }

    pub fn url(&self) -> String {
        let version = self.version();
        match *self {
            Library::Libsodium => {
                let u = format!("https://github.com/jedisct1/libsodium/releases/download/{0}/libsodium-{0}.tar.gz", version);

                u
            },
            Library::Libressl => {
                let u = format!("https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-{}.tar.gz", version);

                u
            },
            Library::Libssh2 => {
                let u = format!("https://www.libssh2.org/download/libssh2-{}.tar.gz", version);

                u
            }
        }
    }

    pub fn products(&self) -> Vec<&str> {
        match *self {
            Library::Libsodium => {
                vec!["libsodium.lib"]
            },
            Library::Libressl => {
                vec!["ssl.lib", "tls.lib", "crypto.lib"]
            },
            Library::Libssh2 => {
                vec!["libssh2.lib"]
            }
        }
    }

    pub fn headers(&self) -> Vec<&str> {
        match *self {
            Library::Libsodium => {
                vec![]
            },
            Library::Libressl => {
                vec!["openssl"]
            },
            Library::Libssh2 => {
                vec!["libssh2.h", "libssh2_sftp.h", "libssh2_publickey.h"]
            }
        }
    }
}