use toolset::Toolset;

use runtime::RuntimeLibrary;

#[derive(Debug, Clone, Copy)]
pub enum Configuration {
    Debug,
    Release,
}

impl From<String> for Configuration {
    fn from(s: String) -> Configuration {
        let lower = s.to_lowercase();
        let r: &str = &lower;
        match r {
            "debug" => Configuration::Debug,
            "release" => Configuration::Release,
            _ => {
                panic!("invalid configuration")
            }
        }
    }
}


impl AsRef<str> for Configuration {
    fn as_ref(&self) -> &str {
        match *self {
            Configuration::Debug => {
                "Debug"
            },
            Configuration::Release => {
                "Release"
            },
        }
    }
}

impl Configuration {
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    pub fn toolset(&self) -> Toolset {
        Toolset::v141_xp
    }

    pub fn runtime_library(&self) -> RuntimeLibrary {
        match *self {
            Configuration::Debug => {
                RuntimeLibrary::MultiThreadedDebugDLL
            },
            Configuration::Release => {
                RuntimeLibrary::MultiThreadedDLL
            },
        }

    }
}