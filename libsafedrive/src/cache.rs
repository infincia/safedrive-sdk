use std::fs::File;
use std::path::{PathBuf, Path};
use std::io::{Read, Write};
use ::rustc_serialize::hex::{FromHex};
use ::walkdir::WalkDir;

use ::CACHE_DIR;
use ::block::WrappedBlock;
use ::error::SDError;

pub fn clean_cache(limit: u64) -> Result<u64, SDError> {
    let cd = CACHE_DIR.read();
    let bp = PathBuf::from(&*cd);

    let mut estimated_size: u64 = 0;
    debug!("estimating size of cache at {}", bp.display());

    let mut delete_candidates: Vec<(PathBuf, u64, ::std::time::SystemTime)> = Vec::new();

    for block in WalkDir::new(&bp).into_iter().filter_map(|e| e.ok()) {
        let p = block.path();
        debug!("estimating size of {}", p.display());

        let md = match ::std::fs::symlink_metadata(&p) {
            Ok(m) => m,
            Err(_) => { continue },
        };

        let stream_length = md.len();
        let created_date = md.created().expect("block has no creation date");

        estimated_size = estimated_size + stream_length;

        delete_candidates.push((p.to_owned(), stream_length, created_date))
    }

    let size = ::util::pretty_bytes(estimated_size as f64);

    if estimated_size > limit {
        debug!("cache is full ({}), cleaning old blocks", size);

        let to_delete = estimated_size - limit;
        let mut deleted = 0;
        delete_candidates.sort_by(|a, b| {
            a.2.cmp(&b.2)
        });
        while deleted < to_delete {
            if let Some(item) = delete_candidates.pop() {
                let size = item.1;
                match ::std::fs::remove_file(item.0) {
                    Ok(()) => deleted += size,
                    Err(e) => {
                        debug!("block could not be deleted: {}", e);
                    }
                }

            }
        }
        debug!("deleted {}", ::util::pretty_bytes(deleted as f64));

        Ok(deleted)

    } else {
        debug!("cache is not full: {}", size);
        debug!("deleted {}", ::util::pretty_bytes(0 as f64));
        Ok(0)
    }
}

pub fn clear_cache() -> Result<u64, SDError> {
    let cd = CACHE_DIR.read();
    let bp = PathBuf::from(&*cd);

    let mut deleted: u64 = 0;
    debug!("estimating size of cache at {}", bp.display());


    for item in WalkDir::new(&bp).into_iter().filter_map(|e| e.ok()) {
        let p = item.path();

        debug!("deleting item from cache {}", p.display());

        let md = match ::std::fs::symlink_metadata(&p) {
            Ok(m) => m,
            Err(_) => { continue },
        };

        let stream_length = md.len();

        match ::std::fs::remove_file(item.path()) {
            Ok(()) => {},
            Err(e) => {
                debug!("item could not be deleted: {}", e);
            }
        }

        deleted += stream_length;

    }
    debug!("deleted {}", ::util::pretty_bytes(deleted as f64));

    Ok(deleted)
}

pub fn read_block<'a>(name: &'a str) -> Result<WrappedBlock, SDError> {
    let cd = CACHE_DIR.read();
    let mut bp = PathBuf::from(&*cd);

    bp.push(name);

    let mut file = try!(File::open(&bp));

    let mut buffer = Vec::new();

    try!(file.read_to_end(&mut buffer));
    let h = name.from_hex().unwrap();

    WrappedBlock::from(buffer, h)

}

pub fn write_binary<'a, T>(item: &T) -> Result<(), SDError> where T: ::binformat::BinaryWriter {
    let cd = CACHE_DIR.read();
    let mut item_path = PathBuf::from(&*cd);
    let name = item.name();

    item_path.push(name);

    let mut f = try!(File::create(&item_path));
    try!(f.write_all(&item.as_binary()));

    Ok(())
}