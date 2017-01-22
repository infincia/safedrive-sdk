use std::fs::File;
use std::path::{PathBuf};
use std::io::{Read, Write};

use ::CACHE_DIR;
use ::models::Block;


pub fn read_block<'a>(name: &'a str) -> Result<Block, ::std::io::Error> {
    let cd = CACHE_DIR.read().unwrap();
    let mut bp = PathBuf::from(&*cd);

    bp.push(name);

    match File::open(&bp) {
        Ok(mut file) => {
            let mut buffer = Vec::new();

            try!(file.read_to_end(&mut buffer));

            Ok(Block { name: name, chunk_data: buffer })
        },
        Err(error) => Err(error),
    }
}

pub fn write_block<'a>(block: &Block) -> Result<(), ::std::io::Error> {
    let cd = CACHE_DIR.read().unwrap();
    let mut bp = PathBuf::from(&*cd);

    bp.push(&block.name);

    let mut f = match File::create(&bp) {
        Ok(file) => file,
        Err(error) => return Err(error),
    };
    try!(f.write_all(&block.chunk_data));

    Ok(())
}