
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Library {
    Libsodium,
    Libressl,
    Libssh2,
    Libiconv,
    Libglib,
    Libdbus,
    Libexpat,
    Libgettext,
    Libffi,
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
            Library::Libiconv => {
                "libiconv"
            },
            Library::Libglib => {
                "libglib"
            },
            Library::Libdbus => {
                "libdbus"
            },
            Library::Libgettext => {
                "libgettext"
            },
            Library::Libffi => {
                "libffi"
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
                "1.0.12"
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
            Library::Libiconv => {
                "1.15"
            },
            Library::Libglib => {
                "2.52.1"
            },
            Library::Libdbus => {
                "1.10.20"
            },
            Library::Libgettext => {
                "0.19.8.1"
            },
            Library::Libffi => {
                "3.2.1"
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
            Library::Libiconv => {
                "iconv_ver"
            },
            Library::Libglib => {
                "glib_ver"
            },
            Library::Libdbus => {
                "dbus_ver"
            },
            Library::Libgettext => {
                "gettext_ver"
            },
            Library::Libffi => {
                "ffi_ver"
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
            Library::Libiconv => {
                false
            },
            Library::Libglib => {
                false
            },
            Library::Libdbus => {
                false
            },
            Library::Libgettext => {
                false
            },
            Library::Libffi => {
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
            Library::Libiconv => {
                false
            },
            Library::Libglib => {
                false
            },
            Library::Libdbus => {
                false
            },
            Library::Libgettext => {
                false
            },
            Library::Libffi => {
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
            Library::Libiconv => {
                let u = format!("https://ftp.gnu.org/pub/gnu/libiconv/libiconv-{}.tar.gz", version);

                u
            },
            Library::Libglib => {
                let u = format!("http://ftp.gnome.org/pub/GNOME/sources/glib/2.52/glib-{}.tar.xz", version);

                u
            },
            Library::Libdbus => {
                let u = format!("https://dbus.freedesktop.org/releases/dbus/dbus-{}.tar.gz", version);

                u
            },
            Library::Libgettext => {
                let u = format!("http://ftp.gnu.org/pub/gnu/gettext/gettext-{}.tar.gz", version);

                u
            },
            Library::Libffi => {
                let u = format!("ftp://sourceware.org/pub/libffi/libffi-{}.tar.gz", version);

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
            Library::Libiconv => {
                vec!["libiconv.lib"]
            },
            Library::Libglib => {
                vec!["libglib-2.0.lib"]
            },
            Library::Libdbus => {
                vec!["libdbus-1.lib"]
            },
            Library::Libgettext => {
                vec!["libgettext.lib"]
            },
            Library::Libffi => {
                vec!["libffi.lib"]
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
            Library::Libiconv => {
                vec![]
            },
            Library::Libglib => {
                vec![]
            },
            Library::Libdbus => {
                vec![]
            },
            Library::Libgettext => {
                vec![]
            },
            Library::Libffi => {
                vec![]
            },
            Library::Zlib => {
                vec![]
            }
        }
    }
}