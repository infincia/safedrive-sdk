#![allow(dead_code)]

use std;
use std::path::{Path};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::cmp::{min, max};

use ::cdc::Chunk;
use ::byteorder::LittleEndian;
use ::byteorder::ByteOrder;

use ::models::SyncVersion;

use ::error::{SDError};

use ::block::{Block};

use ::keys::{Key};

pub struct BlockGeneratorStats {
    pub item_size: u64,
    pub processed_size: u64,
    pub processed_size_compressed: u64,
    pub discovered_chunk_count: u64,
    pub discovered_chunk_size_average: usize,
    pub discovered_chunk_expected_size: usize,
    pub discovered_chunk_smallest_size: usize,
    pub discovered_chunk_largest_size: usize,
    pub discovered_chunk_size_variance: usize,
}

// block abstraction
pub struct BlockGenerator<'a> {
    iter: Box<Iterator<Item=Chunk> + 'a>,
    chunk_file: File,
    main_key: &'a Key,
    hmac_key: &'a Key,
    tweak_key: &'a Key,
    // statistics generation
    item_size: u64,
    processed_size: u64,
    processed_size_compressed: u64,
    chunk_index: u64,
    discovered_chunk_count: u64,
    discovered_chunk_size_average: u64,
    discovered_chunk_expected_size: u64,
    discovered_chunk_smallest_size: u64,
    discovered_chunk_largest_size: u64,
    discovered_chunk_size_variance: i64,
    version: SyncVersion,
}

impl<'a> BlockGenerator<'a> {
    pub fn new(path: &Path, main_key: &'a Key, hmac_key: &'a Key, tweak_key: &'a Key, item_size: u64, version: SyncVersion) -> BlockGenerator<'a> {
        let chunk_file = File::open(path).expect("failed to open file to retrieve chunk");
        let search_file = File::open(path).expect("failed to open file to search for chunks");

        let reader: BufReader<File> = BufReader::new(search_file);

        let byte_iter = reader.bytes().map(|b| b.expect("failed to unwrap block data"));
        let chunk_iter = ChunkGenerator::new(byte_iter, &tweak_key, item_size, version);

        BlockGenerator {
            iter: Box::new(chunk_iter),
            chunk_file: chunk_file,
            main_key: main_key,
            hmac_key: hmac_key,
            tweak_key: tweak_key,
            item_size: item_size,
            processed_size: 0,
            processed_size_compressed: 0,
            chunk_index: 0,
            discovered_chunk_count: 0,
            discovered_chunk_size_average: 0,
            discovered_chunk_expected_size: 1 << version.leading_value_size(),
            discovered_chunk_smallest_size: std::u64::MAX,
            discovered_chunk_largest_size: 0,
            discovered_chunk_size_variance: 0,
            version: version
        }
    }

    pub fn stats(&self) -> BlockGeneratorStats {
        BlockGeneratorStats {
            item_size: self.item_size,
            processed_size: self.processed_size,
            processed_size_compressed: self.processed_size_compressed,
            discovered_chunk_count: self.discovered_chunk_count,
            discovered_chunk_size_average: self.discovered_chunk_size_average as usize,
            discovered_chunk_expected_size: self.discovered_chunk_expected_size as usize,
            discovered_chunk_smallest_size: self.discovered_chunk_smallest_size as usize,
            discovered_chunk_largest_size: self.discovered_chunk_largest_size as usize,
            discovered_chunk_size_variance: self.discovered_chunk_size_variance as usize,
        }
    }
}


impl<'a> Iterator for BlockGenerator<'a> {

    type Item = Result<::block::Block, SDError>;

    fn next(&mut self) -> Option<Result<::block::Block, SDError>> {
        match self.iter.next() {
            Some(chunk) => {
                self.discovered_chunk_count += 1;
                self.processed_size += chunk.size;

                debug!("creating chunk at {} of size {}", self.chunk_index, chunk.size);

                self.discovered_chunk_smallest_size = min(self.discovered_chunk_smallest_size, chunk.size);
                self.discovered_chunk_largest_size = max(self.discovered_chunk_largest_size, chunk.size);
                self.discovered_chunk_size_variance += (chunk.size as i64 - self.discovered_chunk_expected_size as i64).pow(2);
                self.chunk_file.seek(SeekFrom::Start(self.chunk_index)).expect("failed to seek chunk reader");

                let mut buffer = BufReader::new(&self.chunk_file).take(chunk.size);
                let mut data: Vec<u8> = Vec::with_capacity(self.discovered_chunk_expected_size as usize); //expected size of largest chunk

                self.chunk_index = self.chunk_index + chunk.size;

                if let Err(e) = buffer.read_to_end(&mut data) {
                    return Some(Err(SDError::from(e)))
                }

                let block = Block::new(self.version, self.hmac_key, data);

                match block.compressed_size() {
                    Some(size) => {
                        let ratio = (size as f64 / block.real_size() as f64) * 100.0;

                        debug!("generated compressed block, size {}/{} ({}%)", size, block.real_size(), ratio);
                        self.processed_size_compressed += size;
                    },
                    None => {
                        debug!("generated uncompressed block, real size {}", block.real_size());
                    },
                };

                Some(Ok(block))
            },
            None => None,
        }
    }
}


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
                // not using a custom hash yet, but we can
                //let hash = RollingBlake2b::new(tweak_key.clone(), window_size_bits);

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




pub struct RollingBlake2b {
    tweak_key: Key,
    window_size: usize,
    window_size_mask: usize,
    window_data: Vec<u8>,
    window_index: usize,
    pub hash: ::cdc::Polynom64,
    _h: [u8; 8],
}

impl RollingBlake2b {

    pub fn new(tweak: Key, window_size_bits: u32) -> RollingBlake2b {
        let window_size = 1 << window_size_bits;

        let mut window_data = Vec::with_capacity(window_size);
        window_data.resize(window_size, 0);

        RollingBlake2b {
            tweak_key: tweak,
            window_size: window_size,
            window_size_mask: window_size - 1,
            window_data: window_data,
            window_index: 0,
            hash: 0,
            _h: [0; 8],
        }
    }
}

impl ::cdc::RollingHash64 for RollingBlake2b {
    fn reset(&mut self) {
        self.window_data.clear();
        self.window_data.resize(self.window_size, 0);
        self.window_index = 0;
        self.hash = 0;
    }

    fn prefill_window<I>(&mut self, iter: &mut I) -> usize where I: Iterator<Item=u8> {
        let mut nb_bytes_read = 0;

        for _ in 0..(self.window_size)-1 {
            match iter.next() {
                Some(b) => {
                    let _ = self.window_data[self.window_index];
                    self.window_data[self.window_index] = b;

                    self.window_index = (self.window_index + 1) & self.window_size_mask;

                    nb_bytes_read += 1;
                },
                None => break,
            }
        }

        // Because we didn't overwrite that element in the loop above.
        self.window_data[self.window_index] = 0;

        nb_bytes_read
    }

    fn reset_and_prefill_window<I>(&mut self, iter: &mut I) -> usize where I: Iterator<Item=u8> {
        self.reset();

        self.prefill_window(iter)
    }

    fn get_hash(&self) -> &::cdc::Polynom64 {
        &self.hash
    }

    #[inline]
    fn slide(&mut self, byte: &u8) {
        let _ = self.window_data[self.window_index];
        self.window_data[self.window_index] = *byte;

        // blake2-rfc
        // non-SIMD:
        // 2.5MB/s w/128-bit key, 4.35MB/s unkeyed
        // SIMD:
        // 2.94MB/s w/128-bit key, 5MB/s unkeyed
        // let hash = ::blake2_rfc::blake2b::blake2b(8, self.tweak_key.as_blake2_128(), &self.window_data);

        // sodiumoxide auth, 317KB/s
        //let hash = ::sodiumoxide::crypto::auth::authenticate(&self.window_data, &self.tweak_key.as_sodium_auth_key());


        // sodiumoxide blake2b
        // non-SIMD:
        // 1.56MB/s w/128-bit key
        /*unsafe {
            assert!(::libsodium_sys::crypto_generichash(self._h.as_mut_ptr(),
                                                self._h.len(),
                                                self.window_data.as_slice().as_ptr(),
                                                self.window_data.as_slice().len() as u64,
                                                self.tweak_key.as_blake2_128().as_ptr(),
                                                self.tweak_key.as_blake2_128().len()) == 0);
        }
        */

        // sodiumoxide blake2b salted
        // non-SIMD:
        unsafe {
            let personal = [0u8; 16];
            ::libsodium_sys::crypto_generichash_blake2b_salt_personal(self._h.as_mut_ptr(),
                                                     self._h.len(),
                                                     self.window_data.as_slice().as_ptr(),
                                                     self.window_data.as_slice().len() as u64,
                                                     ::std::ptr::null(),
                                                     0,
                                                     self.tweak_key.as_blake2_128().as_ptr() as *const [u8; 16],
                                                     &personal);
        }
        let hash = self._h;

        // blake2b crate, 1.2MB/s
        //let hash = ::blake2b::blake2b_keyed(8, self.tweak_key.as_ref(), &self.window_data);



        // libb2
        /*let mut state: ::libb2_sys::blake2b_state = unsafe { std::mem::uninitialized() };

        let ret = unsafe {
            let key = Some(self.tweak_key.as_blake2_128());

            // streaming API
            match key {
                Some(key) => ::libb2_sys::blake2b_init_key(&mut state,
                                                           8 as ::libc::size_t,
                                                           key.as_ptr() as *const ::libc::c_void,
                                                           key.len() as ::libc::size_t),
                None => ::libb2_sys::blake2b_init(&mut state,
                                                  8 as ::libc::size_t)
            }
        };

        if ret != 0 {
            panic!("blake2b_init returned {}", ret);
        }*/

        /*let ret = unsafe {
            let key = Some(self.tweak_key.as_blake2_128());

            // convenience API
            match key {
                Some(key) => ::libb2_sys::blake2b(self._h.as_mut_ptr() as *mut ::libc::c_uchar,
                                                  self.window_data.as_slice().as_ptr() as *const ::libc::c_void,
                                                  key.as_ptr() as *const ::libc::c_void,
                                                  8 as ::libc::size_t,
                                                  8 as ::libc::size_t,
                                                  key.len() as ::libc::size_t),
                None => ::libb2_sys::blake2b(self._h.as_mut_ptr() as *mut ::libc::c_uchar,
                                             self.window_data.as_slice().as_ptr() as *const ::libc::c_void,
                                             std::ptr::null() as *const ::libc::c_void,
                                             8 as ::libc::size_t,
                                             8 as ::libc::size_t,
                                             0 as ::libc::size_t),
            }
        };

        if ret != 0 {
            panic!("blake2b_init failed: {}", ret);
        }

        let hash = self._h;*/

        // grab the hash value
        let s = hash.as_ref();

        self.hash = LittleEndian::read_u64(&s);

        self.window_index = (self.window_index + 1) & self.window_size_mask;
    }
}