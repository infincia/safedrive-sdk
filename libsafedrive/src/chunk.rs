use ::cdc::Chunk;

use ::models::SyncVersion;

use ::keys::{Key};

// chunk abstraction

pub struct ChunkGenerator<'a> {
    iter: Box<Iterator<Item=Chunk> + 'a>,
    tweak_key: &'a Key,
    version: SyncVersion,
}

impl<'a> ChunkGenerator<'a> {
    pub fn new<I: Iterator<Item=u8> + 'a>(byte_iter: I, tweak_key: &'a Key, total_size: u64, version: SyncVersion) -> ChunkGenerator<'a> {

        let window_size_bits = version.window_size_bits();
        let leading_value_bits = version.leading_value_size();

        let chunk_iter: Box<Iterator<Item=Chunk>> = match version {

            SyncVersion::Version0 => {
                panic!("invalid sync version");
            },

            SyncVersion::Version1 => {

                let separator_iter = ::cdc::SeparatorIter::custom_new(byte_iter, window_size_bits, move |x: u64| {
                    let bit_mask: u64 = (1u64 << leading_value_bits) - 1;

                    x & bit_mask == bit_mask
                });

                let chunk_iter = ::cdc::ChunkIter::new(separator_iter, total_size);

                Box::new(chunk_iter)
            },

            SyncVersion::Version2 => {
                panic!("chunking unimplemented for this sync version");
            },

        };

        ChunkGenerator { iter: chunk_iter, tweak_key: tweak_key, version: version }
    }
}

impl<'a> Iterator for ChunkGenerator<'a> {

    type Item = Chunk;

    fn next(&mut self) -> Option<Chunk> {
        self.iter.next()
    }
}