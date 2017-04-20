#![allow(dead_code)]

use std::path::{PathBuf, Path};
use std::fs::File;

use error::SDError;

#[derive(Debug)]
pub struct FolderLock {
    pub path: PathBuf
}


impl FolderLock {
    pub fn new(folder: &Path) -> Result<FolderLock, SDError> {
        debug!("attempting to lock: {:?}", folder);

        let mut path = PathBuf::from(folder);
        path.push(".sdlock");

        match (&path).exists() {
            true => {
                debug!("lock file says sync already in progress");

                return Err(SDError::SyncAlreadyInProgress);
            },
            false => {
                File::create(&path)?;
            }
        }

        return Ok(FolderLock { path: path })
    }

    pub fn unlock(&self) {
        debug!("dropping lock on {:?}", self.path);

        match ::std::fs::remove_file(&self.path) {
            Ok(()) => {},
            Err(e) => {
                debug!("couldn't drop lock file: {}", e);
            },
        };
    }
}