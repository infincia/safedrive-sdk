#[derive(Debug, Clone, Copy)]
pub enum Toolset {
    v141,
    v141_xp,
}

impl AsRef<str> for Toolset {
    fn as_ref(&self) -> &str {
        match *self {
            Toolset::v141 => {
                "v141"
            },
            Toolset::v141_xp => {
                "v141_xp"
            },
        }
    }
}

impl Toolset {
    pub fn name(&self) -> &str {
        self.as_ref()
    }

    pub fn vs_version(&self) -> &str {
        match *self {
            Toolset::v141 => {
                "Visual Studio 15 2017"
            },
            Toolset::v141_xp => {
                "Visual Studio 15 2017"
            },
        }
    }
}

impl From<String> for Toolset {
    fn from(s: String) -> Toolset {
        let lower = s.to_lowercase();
        let r: &str = &lower;
        match r {
            "v141" => Toolset::v141,
            "v141_xp" => Toolset::v141_xp,
            _ => {
                panic!("invalid toolset")
            }
        }
    }
}
