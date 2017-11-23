
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Library {
    Libsodium,
    Libressl,
    Libssh2,
    Libdbus,
    Libexpat,
    Zlib,
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
            },
            Library::Libexpat => {
                "libexpat"
            },
            Library::Libdbus => {
                "libdbus"
            },
            Library::Zlib => {
                "zlib"
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
                "1.0.14"
            },
            Library::Libressl => {
                "2.5.5"
            },
            Library::Libssh2 => {
                "1.8.0"
            },
            Library::Libexpat => {
                "2.2.2"
            },
            Library::Libdbus => {
                "1.10.20"
            },
            Library::Zlib => {
                "1.2.11"
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
            },
            Library::Libexpat => {
                "expat_ver"
            },
            Library::Libdbus => {
                "dbus_ver"
            },
            Library::Zlib => {
                "z_ver"
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
            },
            Library::Libexpat => {
                false
            },
            Library::Libdbus => {
                false
            },
            Library::Zlib => {
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
            },
            Library::Libexpat => {
                false
            },
            Library::Libdbus => {
                false
            },
            Library::Zlib => {
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
            },
            Library::Libexpat => {
                let u = format!("https://downloads.sourceforge.net/project/expat/expat/{0}/expat-{0}.tar.bz2", version);

                u
            },
            Library::Libdbus => {
                let u = format!("https://dbus.freedesktop.org/releases/dbus/dbus-{}.tar.gz", version);

                u
            },
            Library::Zlib => {
                let u = format!("https://zlib.net/zlib-{}.tar.gz", version);

                u
            }
        }
    }

    pub fn products(&self) -> Vec<&str> {
        match *self {
            Library::Libsodium => {
                vec!["libsodium.lib", "libsodium.pdb"]
            },
            Library::Libressl => {
                vec!["ssl.lib", "tls.lib", "crypto.lib"]
            },
            Library::Libssh2 => {
                vec!["libssh2.lib"]
            },
            Library::Libexpat => {
                vec!["libexpat.lib"]
            },
            Library::Libdbus => {
                vec!["libdbus-1.lib"]
            },
            Library::Zlib => {
                vec!["zlib.lib"]
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
            },
            Library::Libexpat => {
                vec![]
            },
            Library::Libdbus => {
                vec![]
            },
            Library::Zlib => {
                vec![]
            }
        }
    }
}