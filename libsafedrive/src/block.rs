use std::io::{Read, Write};

use ::models::{SyncVersion};

use ::error::{CryptoError, SDError};
use ::binformat::BinaryFormat;
use ::nom::IResult::*;
use ::keys::{Key, WrappedKey, KeyType};
use ::rustc_serialize::hex::{ToHex};
use ::blake2_rfc::blake2b::blake2b;


use ::constants::*;
use ::models::*;
use ::CHANNEL;

#[derive(Debug, Clone)]
pub struct Block {
    version: SyncVersion,
    hmac: Vec<u8>,
    data: Vec<u8>,
    real_size: u64,
    compressed_size: Option<u64>,
    compressed: bool,
    channel: Channel,
    production: bool,
}

impl Block {
    pub fn new(version: SyncVersion, hmac: &Key, data: Vec<u8>) -> Block {

        let real_size = data.len() as u64;

        /// calculate hmac of the block

        let block_hmac = match version {
            SyncVersion::Version1 => {
                let raw_chunk = data.as_slice();

                /// use HMACSHA256
                let hmac_key = hmac.as_sodium_auth_key();

                let tag = ::sodiumoxide::crypto::auth::authenticate(raw_chunk, &hmac_key);

                tag.as_ref().to_vec()
            },
            SyncVersion::Version2 => {
                let raw_chunk = data.as_slice();

                /// use blake2b
                let hmac_key = hmac.as_blake2_256();

                let hash = blake2b(HMAC_SIZE, hmac_key, raw_chunk);

                hash.as_ref().to_vec()
            },
            _ => {
                panic!("Attempted to create invalid block version");
            },
        };

        let (compressed, maybe_compressed_data, maybe_compressed_size) = match version {
            SyncVersion::Version1 => {
                /// no compression

                (false, data, None)
            },
            SyncVersion::Version2 => {
                let buf = Vec::new();
                let mut encoder = ::lz4::EncoderBuilder::new().level(8).build(buf).unwrap();
                encoder.write(data.as_slice()).unwrap();
                let (mut compressed_data, result) = encoder.finish();
                result.unwrap();

                compressed_data.shrink_to_fit();

                let compressed_size = compressed_data.len() as u64;

                // don't use the compressed data if it's not smaller, means lz4 couldn't compress it
                if compressed_size < real_size {
                    (true, compressed_data, Some(compressed_size))
                } else {
                    (false, data, None)
                }
            },
            _ => {
                panic!("Attempted to create invalid session version");
            }
        };


        let c = CHANNEL.read();

        let channel = match *c {
            Channel::Stable => {
                Channel::Stable
            },
            Channel::Beta => {
                Channel::Beta
            },
            Channel::Nightly => {
                Channel::Nightly
            },
        };

        let production = is_production();

        Block { version: version, data: maybe_compressed_data, real_size: real_size, compressed_size: maybe_compressed_size, hmac: block_hmac, compressed: compressed, channel: channel, production: production }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn real_size(&self) -> u64 {
        self.real_size
    }

    pub fn compressed_size(&self) -> Option<u64> {
        self.compressed_size
    }

    pub fn name(&self) -> String {
        self.hmac.to_hex()
    }

    pub fn get_hmac(&self) -> Vec<u8> {
        self.hmac.clone()
    }

    pub fn compressed(&self) -> bool {
        self.compressed
    }

    pub fn production(&self) -> bool {
        self.production
    }

    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    pub fn to_wrapped(self, main: &Key) -> Result<WrappedBlock, CryptoError> {

        /// generate a new block key
        let block_key = Key::new(KeyType::Block);

        /// we use the same nonce both while wrapping the block key, and the block data itself
        /// this is safe because using the same nonce with 2 different keys is not nonce reuse
        let block_nonce = match self.version {
            SyncVersion::Version1 => {
                /// We use the first 24 bytes of the block hmac value as nonce for wrapping
                /// the block key and encrypting the block itself.
                ///
                /// This is cryptographically safe but still deterministic: encrypting
                /// the same block twice with a specific key will always produce the same
                /// output block, which is critical for versioning and deduplication
                /// across all backups of all sync folders
                ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&self.hmac.as_slice()[0..SECRETBOX_NONCE_SIZE as usize])
                    .expect("failed to get nonce")
            },
            SyncVersion::Version2 => {
                /// We use the blake2 hash function to generate exactly 192-bits/24 bytes
                let hash = blake2b(SECRETBOX_NONCE_SIZE, &[], &self.hmac.as_slice());

                ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&hash.as_ref())
                    .expect("failed to get nonce")
            },
            _ => {
                panic!("Attempted to use secretbox nonce with invalid block version");
            },
        };

        /// get the block data, padded and prefixed with a u32 length as little endian
        let to_encrypt = match self.version {
            SyncVersion::Version1 => {
                /// version 1 directly inserts the data before encryption
                self.data
            },
            SyncVersion::Version2 => {
                /// version 2 has padded and prefixed data segments
                ::util::pad_and_prefix_length(self.data.as_slice())
            },
            _ => {
                panic!("Attempted to wrap invalid block version");
            },
        };


        /// encrypt the block data using the block key
        let wrapped_data = ::sodiumoxide::crypto::secretbox::seal(to_encrypt.as_slice(), &block_nonce, &block_key.as_sodium_secretbox_key());

        /// wrap the block key with the main encryption key
        let wrapped_block_key = match block_key.to_wrapped(main, Some(&block_nonce)) {
            Ok(wk) => wk,
            Err(e) => return Err(e),
        };


        Ok(WrappedBlock { version: self.version, hmac: self.hmac, wrapped_data: wrapped_data, wrapped_key: wrapped_block_key, nonce: block_nonce, compressed: self.compressed, channel: self.channel, production: self.production })
    }
}

impl AsRef<[u8]> for Block {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<'a> ::std::fmt::Display for Block {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "Block <version:{}, channel:{}, production:{}, compressed:{}>", self.version, self.channel, self.production, self.compressed)
    }
}

#[derive(Debug, Clone)]
pub struct WrappedBlock {
    version: SyncVersion,
    hmac: Vec<u8>,
    wrapped_data: Vec<u8>,
    nonce: ::sodiumoxide::crypto::secretbox::Nonce,
    wrapped_key: WrappedKey,
    compressed: bool,
    channel: Channel,
    production: bool,
}


impl WrappedBlock {
    pub fn to_block(self, main: &Key) -> Result<Block, SDError> {

        let block_key = match self.wrapped_key.to_key(main, Some(&self.nonce)) {
            Ok(k) => k,
            Err(e) => {
                debug!("block key unwrap failed: {:?}", e);
                return Err(SDError::CryptoError(CryptoError::BlockDecryptFailed))
            }
        };

        let block_raw = match ::sodiumoxide::crypto::secretbox::open(&self.wrapped_data, &self.nonce, &block_key.as_sodium_secretbox_key()) {
            Ok(s) => s,
            Err(e) => {
                debug!("block decrypt failed: {:?}", e);
                return Err(SDError::CryptoError(CryptoError::BlockDecryptFailed))
            }
        };

        let unpadded_data = match self.version {
            SyncVersion::Version1 => {
                block_raw
            },
            SyncVersion::Version2 => {
                let unpadded = match ::binformat::remove_padding(&block_raw) {
                    Done(_, o) => o,
                    Error(e) => {
                        debug!("block padding removal failed: {}", &e);

                        match e {
                            ::nom::verbose_errors::Err::Code(ref kind) => {
                                debug!("block padding removal failure kind: {:?}", kind);

                            },
                            ::nom::verbose_errors::Err::Node(ref kind, ref err) => {
                                debug!("block padding removal failure node: {:?}, {}", kind, err);

                            },
                            ::nom::verbose_errors::Err::Position(ref kind, ref position) => {
                                debug!("block padding removal failure position: {:?}: {:?}", kind, position);

                            },
                            ::nom::verbose_errors::Err::NodePosition(ref kind, ref position, ref err) => {
                                debug!("block padding removal failure kind: {:?}: {:?}, {}", kind, position, err);

                            },
                        };

                        return Err(SDError::BlockUnreadable) },
                    Incomplete(_) => {
                        debug!("block data cannot be unpadded, this should never happen");
                        return Err(SDError::BlockUnreadable)
                    }
                };

                let unpadded_data = unpadded.to_vec();

                unpadded_data
            },
            _ => panic!("unknown binary version")
        };

        let (maybe_uncompressed_data, maybe_compressed_size) = match self.version {
            SyncVersion::Version1 => {
                (unpadded_data, None)
            },
            SyncVersion::Version2 => {
                match self.compressed {
                    true => {
                        let compressed_size = unpadded_data.len() as u64;

                        let mut uncompressed_data = Vec::new();
                        let mut decoder = ::lz4::Decoder::new(unpadded_data.as_slice()).unwrap();
                        let _ = decoder.read_to_end(&mut uncompressed_data);

                        (uncompressed_data, Some(compressed_size))
                    },
                    false => {
                        (unpadded_data, None)
                    }
                }
            },
            _ => panic!("unknown binary version")
        };

        let real_size = maybe_uncompressed_data.len();


        Ok(Block { version: self.version, hmac: self.hmac, data: maybe_uncompressed_data, real_size: real_size as u64, compressed_size: maybe_compressed_size, compressed: self.compressed, channel: self.channel, production: self.production })
    }

    pub fn get_hmac(&self) -> Vec<u8> {
        self.hmac.clone()
    }

    pub fn compressed(&self) -> bool {
        self.compressed
    }

    pub fn len(&self) -> usize {
        self.wrapped_data.len()
    }

    pub fn real_size(&self) -> u64 {
        panic!("can't retrieve real size of a wrapped block");
    }

    pub fn to_binary(&self) -> Vec<u8> {
        let mut binary_data = Vec::new();

        /// first 8 bytes are the file ID, type, version, flags, and 2 byte reserved area
        let magic: &'static [u8; 2] = br"sd";
        let file_type: &'static [u8; 1] = br"b";
        let version = self.version.as_ref();
        let reserved: &'static [u8; 2] = br"00";

        let mut flags = Empty;

        match self.channel {
            Channel::Stable => {
                flags.insert(Stable)
            },
            Channel::Beta => {
                flags.insert(Beta);
            },
            Channel::Nightly => {
                flags.insert(Nightly);
            },
        };

        if self.production {
            flags.insert(Production);
        } else {
        }

        if self.compressed() {
            flags.insert(Compressed);
        } else {
        }

        let flag_ref: &[u8] = &[flags.bits()];

        binary_data.extend(magic.as_ref());
        binary_data.extend(file_type.as_ref());
        binary_data.extend(version);
        binary_data.extend(flag_ref);
        binary_data.extend(reserved.as_ref());

        /// next 48 bytes will be the wrapped block key
        binary_data.extend(self.wrapped_key.as_ref());

        /// next 24 bytes will be the nonce
        let n: &[u8] = self.nonce.as_ref();
        binary_data.extend(n);
        assert!(binary_data.len() == magic.len() + file_type.len() + version.len() + flag_ref.len() + reserved.len() + (SECRETBOX_KEY_SIZE + SECRETBOX_MAC_SIZE) + SECRETBOX_NONCE_SIZE);

        /// remainder will be the encrypted block data
        binary_data.extend(self.wrapped_data.as_slice());

        binary_data
    }

    pub fn from(raw: Vec<u8>, hmac: Vec<u8>) -> Result<WrappedBlock, SDError> {

        let raw_block: BinaryFormat = match ::binformat::binary_parse(&raw) {
            Done(_, o) => o,
            Error(e) => {
                debug!("block parsing failed: {}", &e);

                match e {
                    ::nom::verbose_errors::Err::Code(ref kind) => {
                        debug!("block parse failure kind: {:?}", kind);

                    },
                    ::nom::verbose_errors::Err::Node(ref kind, ref err) => {
                        debug!("block parse failure node: {:?}, {}", kind, err);

                    },
                    ::nom::verbose_errors::Err::Position(ref kind, ref position) => {
                        debug!("block parse failure position: {:?}: {:?}", kind, position);

                    },
                    ::nom::verbose_errors::Err::NodePosition(ref kind, ref position, ref err) => {
                        debug!("block parse failure kind: {:?}: {:?}, {}", kind, position, err);

                    },
                };

                return Err(SDError::BlockUnreadable)
            },
            Incomplete(_) => {
                debug!("block file cannot be parsed, this should never happen");
                return Err(SDError::BlockUnreadable)
            }
        };

        debug!("got valid binary file: {}", &raw_block);

        let block_ver = match raw_block.version {
            "01" => SyncVersion::Version1,
            "02" => SyncVersion::Version2,
            _ => panic!("unknown binary version")
        };
        let wrapped_block_key_raw = raw_block.wrapped_key.to_vec();
        let nonce_raw = raw_block.nonce;

        let block_nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_raw)
            .expect("failed to get nonce");

        let wrapped_block_raw = raw_block.wrapped_data.to_vec();

        let wrapped_block_key = WrappedKey::from(wrapped_block_key_raw, KeyType::Block);

        let channel = raw_block.channel;

        let production = raw_block.production;
        let compressed = raw_block.compressed;

        let wrapped_block = WrappedBlock { version: block_ver, hmac: hmac, wrapped_key: wrapped_block_key, wrapped_data: wrapped_block_raw, nonce: block_nonce, compressed: compressed, channel: channel, production: production };
        debug!("got valid wrapped block: {}", &wrapped_block);

        Ok(wrapped_block)
    }

}

impl<'a> ::std::fmt::Display for WrappedBlock {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "WrappedBlock <version:{}, channel:{}, production:{}, compressed:{}>", self.version, self.channel, self.production, self.compressed)
    }
}

#[allow(dead_code)]
static TEST_BLOCK_DATA_UNENCRYPTED: [u8; 1024] = [7u8; 1024];

#[test]
fn new_block_v1_test() {
    let hmac = Key::new(KeyType::HMAC);
    let test_data = Vec::from(TEST_BLOCK_DATA_UNENCRYPTED.as_ref());

    let _ = Block::new(SyncVersion::Version1, &hmac, test_data);
}

#[test]
fn new_block_v2_test() {
    let hmac = Key::new(KeyType::HMAC);
    let test_data = Vec::from(TEST_BLOCK_DATA_UNENCRYPTED.as_ref());

    let _ = Block::new(SyncVersion::Version2, &hmac, test_data);
}

#[test]
fn wrap_block_v1_test() {
    let main = Key::new(KeyType::Main);
    let hmac = Key::new(KeyType::HMAC);

    let test_data = Vec::from(TEST_BLOCK_DATA_UNENCRYPTED.as_ref());

    let block = Block::new(SyncVersion::Version1, &hmac, test_data);


    let _ = match block.to_wrapped(&main) {
        Ok(wb) => wb,
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn wrap_block_v2_test() {
    let main = Key::new(KeyType::Main);
    let hmac = Key::new(KeyType::HMAC);

    let test_data = Vec::from(TEST_BLOCK_DATA_UNENCRYPTED.as_ref());

    let block = Block::new(SyncVersion::Version2, &hmac, test_data);


    let _ = match block.to_wrapped(&main) {
        Ok(wb) => wb,
        Err(_) => { assert!(true == false); return }
    };
}


#[test]
fn unwrap_block_v1_test() {
    let main = Key::new(KeyType::Main);
    let hmac = Key::new(KeyType::HMAC);

    let test_data = Vec::from(TEST_BLOCK_DATA_UNENCRYPTED.as_ref());

    let block = Block::new(SyncVersion::Version1, &hmac, test_data);


    let wrapped_block = match block.to_wrapped(&main) {
        Ok(wb) => wb,
        Err(_) => { assert!(true == false); return }
    };

    let _ = match wrapped_block.to_block(&main) {
        Ok(uwb) => uwb,
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn unwrap_block_v2_test() {
    let main = Key::new(KeyType::Main);
    let hmac = Key::new(KeyType::HMAC);

    let test_data = Vec::from(TEST_BLOCK_DATA_UNENCRYPTED.as_ref());

    let block = Block::new(SyncVersion::Version2, &hmac, test_data);


    let wrapped_block = match block.to_wrapped(&main) {
        Ok(wb) => wb,
        Err(_) => { assert!(true == false); return }
    };

    let _ = match wrapped_block.to_block(&main) {
        Ok(uwb) => uwb,
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn wrapped_block_from_vec_v1_test() {
    let main = Key::new(KeyType::Main);
    let hmac = Key::new(KeyType::HMAC);

    let test_data = Vec::from(TEST_BLOCK_DATA_UNENCRYPTED.as_ref());

    let block = Block::new(SyncVersion::Version1, &hmac, test_data);
    let hmac_value = block.get_hmac();

    let wrapped_block = match block.to_wrapped(&main) {
        Ok(wb) => wb,
        Err(_) => { assert!(true == false); return }
    };

    let raw_wrapped_data = wrapped_block.to_binary();


    let _ = match WrappedBlock::from(raw_wrapped_data, hmac_value.to_vec()) {
        Ok(rwb) => rwb,
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn wrapped_block_from_vec_v2_test() {
    let main = Key::new(KeyType::Main);
    let hmac = Key::new(KeyType::HMAC);

    let test_data = Vec::from(TEST_BLOCK_DATA_UNENCRYPTED.as_ref());

    let block = Block::new(SyncVersion::Version2, &hmac, test_data);
    let hmac_value = block.get_hmac();

    let wrapped_block = match block.to_wrapped(&main) {
        Ok(wb) => wb,
        Err(_) => { assert!(true == false); return }
    };

    let raw_wrapped_data = wrapped_block.to_binary();


    let _ = match WrappedBlock::from(raw_wrapped_data, hmac_value.to_vec()) {
        Ok(rwb) => rwb,
        Err(_) => { assert!(true == false); return }
    };
}