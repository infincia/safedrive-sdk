#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Platform {
    i686,
    x86_64,
}

impl From<String> for Platform {
    fn from(s: String) -> Platform {
        let lower = s.to_lowercase();
        let r: &str = &lower;
        match r {
            "win32" => Platform::i686,
            "x64" => Platform::x86_64,
            _ => {
                panic!("invalid platform")
            }
        }
    }
}

impl Platform {
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    pub fn target(&self) -> &str {
        match *self {
            Platform::i686 => {
                "i686-pc-windows-msvc"
            },
            Platform::x86_64 => {
                "x86_64-pc-windows-msvc"
            }
        }
    }
}

impl AsRef<str> for Platform {
    fn as_ref(&self) -> &str {
        match *self {
            Platform::i686 => {
                "Win32"
            },
            Platform::x86_64 => {
                "x64"
            },
        }
    }
}