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
        let min_chunk_size = version.min_chunk_size();
        let max_chunk_size = version.max_chunk_size();

        let chunk_iter: Box<Iterator<Item=Chunk>> = match version {

            SyncVersion::Version0 => {
                panic!("invalid sync version");
            },

            SyncVersion::Version1 => {
                let hash = ::cdc::Rabin64::new(window_size_bits);

                let separator_iter = SeparatorIter::custom_new(byte_iter, min_chunk_size, max_chunk_size, hash, move |x: u64| {
                    let bit_mask: u64 = (1u64 << leading_value_bits) - 1;

                    x & bit_mask == bit_mask
                });

                let chunk_iter = ::cdc::ChunkIter::new(separator_iter, total_size);

                Box::new(chunk_iter)
            },

            SyncVersion::Version2 => {
                let hash = ::cdc::Rabin64::new(window_size_bits);

                let separator_iter = SeparatorIter::custom_new(byte_iter, min_chunk_size, max_chunk_size, hash, move |x: u64| {
                    let bit_mask: u64 = (1u64 << leading_value_bits) - 1;

                    x & bit_mask == bit_mask
                });

                let chunk_iter = ::cdc::ChunkIter::new(separator_iter, total_size);

                Box::new(chunk_iter)
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




// sep

pub struct SeparatorIter<I, F, H> {
    iter: I,
    predicate: F,
    hash: H,
    index: u64,
    min_chunk_size: usize,
    max_chunk_size: usize,
}

impl<I, F, H> SeparatorIter<I, F, H> where I: Iterator<Item=u8>, F: Fn(u64) -> bool, H: ::cdc::RollingHash64 {
    pub fn custom_new(mut iter: I, min_chunk_size: usize, max_chunk_size: usize, hash: H, predicate: F) -> SeparatorIter<I, F, H> {
        let mut local_hash = hash;

        let index = local_hash.reset_and_prefill_window(&mut iter) as u64;

        SeparatorIter {
            iter: iter,
            predicate: predicate,
            hash: local_hash,
            index: index,
            min_chunk_size: min_chunk_size,
            max_chunk_size: max_chunk_size,
        }
    }
}

impl<I, F, H> Iterator for SeparatorIter<I, F, H> where I: Iterator<Item=u8>, F: Fn(u64) -> bool, H: ::cdc::RollingHash64 {
    type Item = ::cdc::Separator;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(byte) = self.iter.next() {
            self.hash.slide(&byte);
            self.index += 1;
            if (self.predicate)(*self.hash.get_hash()) {
                let separator = ::cdc::Separator { index: self.index, hash: *self.hash.get_hash() };

                // Note: We skip min chunk size + subsequent separators which may overlap the current one.
                self.index += self.hash.reset_and_prefill_window(&mut self.iter) as u64;

                return Some(separator);
            }
        }

        None
    }

}


}