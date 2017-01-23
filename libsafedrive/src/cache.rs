use std::fs::File;
use std::path::{PathBuf};
use std::io::{Read, Write};
use ::rustc_serialize::hex::{ToHex, FromHex};

use ::CACHE_DIR;
use ::block::WrappedBlock;
use ::error::SDError;


pub fn read_block<'a>(name: &'a str) -> Result<WrappedBlock, SDError> {
    let cd = CACHE_DIR.read().unwrap();
    let mut bp = PathBuf::from(&*cd);

    bp.push(name);

    let mut file = try!(File::open(&bp));

    let mut buffer = Vec::new();

    try!(file.read_to_end(&mut buffer));
    let h = name.from_hex().unwrap();

    WrappedBlock::from(buffer, h)

}

pub fn write_block<'a>(block: &WrappedBlock) -> Result<(), SDError> {
    let cd = CACHE_DIR.read().unwrap();
    let mut bp = PathBuf::from(&*cd);
    let h = block.hmac.to_hex();

    bp.push(h);

    let mut f = try!(File::create(&bp));
    try!(f.write_all(&block.to_binary()));

    Ok(())
}