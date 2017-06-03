use std;
use subprocess;
use fs_extra;

#[derive(Debug)]
pub enum BuildError {
    Error(Box<::std::error::Error>),
    PopDirectoryError,
    CommandFailed(String),
    CopyFailed(fs_extra::error::Error),
    SourceMissing(String),
    BuiltProductMissing(String),
    BuildFailed(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            BuildError::Error(ref err) => write!(f, "{}", err),
            BuildError::PopDirectoryError => write!(f, "PopDirectoryError"),
            BuildError::CommandFailed(ref string) => write!(f, "command failed: {}", string),
            BuildError::CopyFailed(ref err) => write!(f, "{}", err),
            BuildError::SourceMissing(ref string) => write!(f, "source missing: {}", string),
            BuildError::BuiltProductMissing(ref string) => write!(f, "built product missing: {}", string),
            BuildError::BuildFailed(ref string) => write!(f, "build failed: {}", string),
        }
    }
}


impl std::error::Error for BuildError {
    fn description(&self) -> &str {
        match *self {
            BuildError::Error(ref err) => err.description(),
            BuildError::PopDirectoryError => "pop directory error",
            BuildError::CommandFailed(_) => "command failed",
            BuildError::CopyFailed(_) => "copy failed",
            BuildError::SourceMissing(_) => "source missing",
            BuildError::BuiltProductMissing(_) => "built product missing",
            BuildError::BuildFailed(_) => "build failed",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            BuildError::Error(ref err) => Some(&**err),
            BuildError::PopDirectoryError => None,
            BuildError::CommandFailed(_) => None,
            BuildError::CopyFailed(ref err) => Some(err),
            BuildError::SourceMissing(_) => None,
            BuildError::BuiltProductMissing(_) => None,
            BuildError::BuildFailed(_) => None,

        }
    }
}

impl From<subprocess::PopenError> for BuildError {
    fn from(err: subprocess::PopenError) -> BuildError {
        match err {
            _ => BuildError::CommandFailed(format!("{}", err)),
        }
    }
}

impl From<std::io::Error> for BuildError {
    fn from(err: std::io::Error) -> BuildError {
        match err {
            _ => BuildError::Error(Box::new(err)),
        }
    }
}

impl From<fs_extra::error::Error> for BuildError {
    fn from(err: fs_extra::error::Error) -> BuildError {
        match err {
            _ => BuildError::CopyFailed(err),
        }
    }
}
