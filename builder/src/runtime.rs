#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum RuntimeLibrary {
    MultiThreadedDebug,
    MultiThreaded,
    MultiThreadedDebugDLL,
    MultiThreadedDLL,
}

impl RuntimeLibrary {

    pub fn name(&self) -> &str {
        self.as_ref()
    }

    pub fn cxx_flags_release(&self) -> &str {
        "/MD"
    }

    pub fn cxx_flags_debug(&self) -> &str {
        "/MDd"
    }

    pub fn c_flags_release(&self) -> &str {
        "/MD"
    }

    pub fn c_flags_debug(&self) -> &str {
        "/MDd"
    }
}


impl AsRef<str> for RuntimeLibrary {
    fn as_ref(&self) -> &str {
        match *self {
            RuntimeLibrary::MultiThreadedDLL => {
                "/MD"
            },
            RuntimeLibrary::MultiThreadedDebugDLL => {
                "/MDd"
            },
            RuntimeLibrary::MultiThreaded => {
                "/MT"
            },
            RuntimeLibrary::MultiThreadedDebug => {
                "/MTd"
            },
        }
    }
}
