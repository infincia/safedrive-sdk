#![cfg(feature = "sessionfs")]

use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::path::{Path, PathBuf};
use time::Timespec;

use libc;

use fuse_mt::*;

use ::models::{RegisteredFolder};
use ::session::SyncSession;

pub struct SessionFS {
    pub target: OsString,
    pub folders: Vec<RegisteredFolder>,
}

impl SessionFS {
    pub fn mount(self, path: PathBuf) {
        let fuse_args: Vec<&OsStr> = vec![&OsStr::new("-o"), &OsStr::new("auto_unmount")];

        mount(FuseMT::new(self, 1), &path, &fuse_args).unwrap();
    }
}


impl FilesystemMT for SessionFS {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        debug!("init");
        Ok(())
    }

    fn destroy(&self, _req: RequestInfo) {
        debug!("destroy");
    }

    fn lookup(&self, _req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEntry {
        debug!("lookup: {:?}/{:?}", parent, name);

        let root = PathBuf::from("/");

        if parent != &root {
            return Err(libc::EINVAL)
        }

        let f_s: &[RegisteredFolder] = self.folders.as_slice();
        let f_i = f_s.into_iter();

        let f_res = f_i.filter(|f| {
            let ref f_name: String = f.folderName;
            let f_o_name: &OsStr = f_name.as_ref();

            f_o_name == f_o_name
        });

        let f: Option<&RegisteredFolder> = f_res.last();

        match f {
            Some(f) => {
                let t = Timespec { sec: 1 as i64, nsec: 0 as i32 };

                let a = FileAttr {
                    ino: f.id as u64,
                    size: 0 as u64,
                    blocks: 0 as u64,
                    atime: Timespec { sec: f.addedDate as i64, nsec: 0 as i32 },
                    mtime: Timespec { sec: f.addedDate as i64, nsec: 0 as i32 },
                    ctime: Timespec { sec: f.addedDate as i64, nsec: 0 as i32 },
                    crtime: Timespec { sec: f.addedDate as i64, nsec: 0 },
                    kind: FileType::Directory,
                    perm: 0o7777 as u16,
                    nlink: 1 as u32,
                    uid: 501,
                    gid: 20,
                    rdev: 0 as u32,
                    flags: 0,
                };

                Ok((t, a))
            },
            None => {
                return Err(libc::EINVAL)
            }
        }
    }

    fn getattr(&self, _req: RequestInfo, _path: &Path, _fh: Option<u64>) -> ResultGetattr {
        debug!("getattr: {:?}", _path);
        let ttl = Timespec { sec: 1 as i64, nsec: 0 as i32 };

        let root = PathBuf::from("/");
        let root_p: &Path = root.as_ref();

        if _path == root_p {
            let a = FileAttr {
                ino: 0,
                size: self.folders.len() as u64,
                blocks: 0 as u64,
                atime: Timespec { sec: 0 as i64, nsec: 0 as i32 },
                mtime: Timespec { sec: 0 as i64, nsec: 0 as i32 },
                ctime: Timespec { sec: 0 as i64, nsec: 0 as i32 },
                crtime: Timespec { sec: 0, nsec: 0 },
                kind: FileType::Directory,
                perm: 0o7777 as u16,
                nlink: 1 as u32,
                uid: 501,
                gid: 20,
                rdev: 0 as u32,
                flags: 0,
            };

            return Ok((ttl, a));

        }

        let relative_path = _path.strip_prefix(&root).expect("failed to unwrap relative path");


        let f_s: &[RegisteredFolder] = self.folders.as_slice();
        let f_i = f_s.into_iter();


        let f_res = f_i.filter(|f| {
            let ref f_name: String = f.folderName;
            let f_o_name: &OsStr = f_name.as_ref();

            f_o_name == relative_path.file_name().expect("failed to get file name")
        });


        let f: Option<&RegisteredFolder> = f_res.last();

        match f {
            Some(f) => {
                let t = Timespec { sec: 1 as i64, nsec: 0 as i32 };

                let a = FileAttr {
                    ino: f.id as u64,
                    size: 0 as u64,
                    blocks: 0 as u64,
                    atime: Timespec { sec: f.addedDate as i64, nsec: 0 as i32 },
                    mtime: Timespec { sec: f.addedDate as i64, nsec: 0 as i32 },
                    ctime: Timespec { sec: f.addedDate as i64, nsec: 0 as i32 },
                    crtime: Timespec { sec: f.addedDate as i64, nsec: 0 },
                    kind: FileType::Directory,
                    perm: 0o7777 as u16,
                    nlink: 1 as u32,
                    uid: 501,
                    gid: 20,
                    rdev: 0 as u32,
                    flags: 0,
                };

                Ok((t, a))
            },
            None => {
                return Err(libc::EINVAL)
            }
        }




    }

    fn opendir(&self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        debug!("opendir: {:?} (flags = {:#o})", path, _flags);
        Ok((5, 0))

    }

    fn releasedir(&self, _req: RequestInfo, path: &Path, fh: u64, _flags: u32) -> ResultEmpty {
        debug!("releasedir: {:?}", path);
        Ok(())
    }

    fn readdir(&self, _req: RequestInfo, path: &Path, fh: u64) -> ResultReaddir {
        debug!("readdir: {:?}", path);
        let mut entries: Vec<DirectoryEntry> = vec![];

        if fh == 0 {
            error!("readdir: missing fh");
            return Err(libc::EINVAL);
        }

        let root = PathBuf::from("/");
        let rootref: &Path = root.as_ref();

        if path == rootref {
            let f_s: &[RegisteredFolder] = self.folders.as_slice();
            let f_i = f_s.into_iter();

            for folder in f_s {
                let ref f_name: String = folder.folderName;
                let f_o_name: &OsStr = f_name.as_ref();

                entries.push(DirectoryEntry {
                    name: f_o_name.to_owned(),
                    kind: FileType::Directory,
                })
            }
        }

        Ok(entries)
    }

    fn open(&self, _req: RequestInfo, path: &Path, flags: u32) -> ResultOpen {
        debug!("open: {:?} flags={:#x}", path, flags);
        Err(libc::EINVAL)
    }

    fn release(&self, _req: RequestInfo, path: &Path, fh: u64, _flags: u32, _lock_owner: u64, _flush: bool) -> ResultEmpty {
        debug!("release: {:?}", path);
        Ok(())
    }

    fn read(&self, _req: RequestInfo, path: &Path, fh: u64, offset: u64, size: u32) -> ResultData {
        debug!("read: {:?} {:#x} @ {:#x}", path, size, offset);
        Err(libc::EINVAL)
    }

    fn write(&self, _req: RequestInfo, path: &Path, fh: u64, offset: u64, data: Vec<u8>, _flags: u32) -> ResultWrite {
        debug!("write: {:?} {:#x} @ {:#x}", path, data.len(), offset);
        Err(libc::EINVAL)
    }

    fn flush(&self, _req: RequestInfo, path: &Path, fh: u64, _lock_owner: u64) -> ResultEmpty {
        debug!("flush: {:?}", path);
        Ok(())
    }

    fn fsync(&self, _req: RequestInfo, path: &Path, fh: u64, datasync: bool) -> ResultEmpty {
        debug!("fsync: {:?}, data={:?}", path, datasync);
        Ok(())
    }

    fn chmod(&self, _req: RequestInfo, path: &Path, fh: Option<u64>, mode: u32) -> ResultEmpty {
        debug!("chmod: {:?} to {:#o}", path, mode);
        Ok(())
    }

    fn chown(&self, _req: RequestInfo, path: &Path, fh: Option<u64>, uid: Option<u32>, gid: Option<u32>) -> ResultEmpty {
        debug!("chown: {:?}", path);

        Ok(())
    }

    fn truncate(&self, _req: RequestInfo, path: &Path, fh: Option<u64>, size: u64) -> ResultEmpty {
        debug!("truncate: {:?} to {:#x}", path, size);
        Ok(())
    }

    fn utimens(&self, _req: RequestInfo, path: &Path, fh: Option<u64>, atime: Option<Timespec>, mtime: Option<Timespec>) -> ResultEmpty {
        debug!("utimens: {:?}: {:?}, {:?}", path, atime, mtime);
        Ok(())
    }

    fn readlink(&self, _req: RequestInfo, path: &Path) -> ResultData {
        debug!("readlink: {:?}", path);
        Err(libc::EINVAL)
    }

    fn statfs(&self, _req: RequestInfo, path: &Path) -> ResultStatfs {
        debug!("statfs: {:?}", path);
        Ok(Statfs {
            blocks: 0,
            bfree: 0,
            bavail: 0,
            files: 5,
            ffree: 0,
            bsize: 0 as u32,
            namelen: 0, // TODO
            frsize: 0, // TODO
        })
    }

    fn fsyncdir(&self, _req: RequestInfo, path: &Path, fh: u64, datasync: bool) -> ResultEmpty {
        debug!("fsyncdir: {:?} (datasync = {:?})", path, datasync);
        Ok(())
    }

    fn mknod(&self, _req: RequestInfo, parent_path: &Path, name: &OsStr, mode: u32, rdev: u32) -> ResultEntry {
        debug!("mknod: {:?}/{:?} (mode={:#o}, rdev={})", parent_path, name, mode, rdev);
        Err(libc::EINVAL)
    }

    fn mkdir(&self, _req: RequestInfo, parent_path: &Path, name: &OsStr, mode: u32) -> ResultEntry {
        debug!("mkdir {:?}/{:?} (mode={:#o})", parent_path, name, mode);
        Err(libc::EINVAL)
    }

    fn unlink(&self, _req: RequestInfo, parent_path: &Path, name: &OsStr) -> ResultEmpty {
        debug!("unlink {:?}/{:?}", parent_path, name);
        Ok(())
    }

    fn rmdir(&self, _req: RequestInfo, parent_path: &Path, name: &OsStr) -> ResultEmpty {
        debug!("rmdir: {:?}/{:?}", parent_path, name);
        Ok(())
    }

    fn symlink(&self, _req: RequestInfo, parent_path: &Path, name: &OsStr, target: &Path) -> ResultEntry {
        debug!("symlink: {:?}/{:?} -> {:?}", parent_path, name, target);
        Err(libc::EINVAL)
    }

    fn rename(&self, _req: RequestInfo, parent_path: &Path, name: &OsStr, newparent_path: &Path, newname: &OsStr) -> ResultEmpty {
        debug!("rename: {:?}/{:?} -> {:?}/{:?}", parent_path, name, newparent_path, newname);
        Ok(())
    }

    fn link(&self, _req: RequestInfo, path: &Path, newparent: &Path, newname: &OsStr) -> ResultEntry {
        debug!("link: {:?} -> {:?}/{:?}", path, newparent, newname);
        Err(libc::EINVAL)
    }

    fn create(&self, _req: RequestInfo, parent: &Path, name: &OsStr, mode: u32, flags: u32) -> ResultCreate {
        debug!("create: {:?}/{:?} (mode={:#o}, flags={:#x})", parent, name, mode, flags);
        Err(libc::EINVAL)
    }

    fn listxattr(&self, _req: RequestInfo, path: &Path, size: u32) -> ResultXattr {
        debug!("listxattr: {:?}", path);
        Err(libc::EINVAL)
    }

    fn getxattr(&self, _req: RequestInfo, path: &Path, name: &OsStr, size: u32) -> ResultXattr {
        debug!("getxattr: {:?} {:?} {}", path, name, size);
        Err(libc::EINVAL)
    }

    fn setxattr(&self, _req: RequestInfo, path: &Path, name: &OsStr, value: &[u8], flags: u32, position: u32) -> ResultEmpty {
        debug!("setxattr: {:?} {:?} {} bytes, flags = {:#x}, pos = {}", path, name, value.len(), flags, position);
        Err(libc::EINVAL)
    }

    fn removexattr(&self, _req: RequestInfo, path: &Path, name: &OsStr) -> ResultEmpty {
        debug!("removexattr: {:?} {:?}", path, name);
        Err(libc::EINVAL)
    }
}
