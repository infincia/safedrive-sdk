
use ::models::{SyncVersion};





#[derive(Debug)]
pub struct Block<'a> {
    pub chunk_data: Vec<u8>,
    pub name: &'a str
}

