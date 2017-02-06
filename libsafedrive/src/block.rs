
use ::models::{SyncVersion};

use ::error::{CryptoError, SDError};
use ::binformat::BinaryFormat;
use ::nom::IResult::*;
use ::keys::{Key, WrappedKey, KeyType};
use ::rustc_serialize::hex::{ToHex};
use ::blake2_rfc::blake2b::blake2b;


use ::constants::*;


#[derive(Debug, Clone)]
pub struct Block {
    version: SyncVersion,
    hmac: Vec<u8>,
    data: Vec<u8>
}

impl Block {
    pub fn new(version: SyncVersion, hmac: &Key, data: Vec<u8>) -> Block {
        // calculate hmac of the block

        let block_hmac = match version {
            SyncVersion::Version1 => {
                let raw_chunk = data.as_slice();

                // use HMACSHA256
                let hmac_key = hmac.as_sodium_auth_key();

                let tag = ::sodiumoxide::crypto::auth::authenticate(raw_chunk, &hmac_key);

                tag.as_ref().to_vec()
            },
            SyncVersion::Version2 => {
                let raw_chunk = data.as_slice();

                // use blake2b
                let hmac_key = hmac.as_ref();

                let hash = blake2b(HMAC_SIZE, hmac_key, raw_chunk);

                hash.as_ref().to_vec()
            },
            _ => {
                panic!("Attempted to create invalid block version");
            },
        };

        Block { version: version, data: data, hmac: block_hmac }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn name(&self) -> String {
        self.hmac.to_hex()
    }

    pub fn get_hmac(&self) -> Vec<u8> {
        self.hmac.clone()
    }

    pub fn to_wrapped(self, main: &Key) -> Result<WrappedBlock, CryptoError> {

        // generate a new block key
        let block_key = Key::new(KeyType::Block);

        // we use the same nonce both while wrapping the block key, and the block data itself
        // this is safe because using the same nonce with 2 different keys is not nonce reuse
        let block_nonce = match self.version {
            SyncVersion::Version1 => {
                // We use the first 24 bytes of the block hmac value as nonce for wrapping
                // the block key and encrypting the block itself.
                //
                // This is cryptographically safe but still deterministic: encrypting
                // the same block twice with a specific key will always produce the same
                // output block, which is critical for versioning and deduplication
                // across all backups of all sync folders
                ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&self.hmac.as_slice()[0..SECRETBOX_NONCE_SIZE as usize])
                    .expect("failed to get nonce")
            },
            SyncVersion::Version2 => {
                // We use the blake2 hash function to generate exactly 192-bits/24 bytes
                let hash = blake2b(SECRETBOX_NONCE_SIZE, &[], &self.hmac.as_slice());

                ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&hash.as_ref())
                    .expect("failed to get nonce")
            },
            _ => {
                panic!("Attempted to use secretbox nonce with invalid block version");
            },
        };

        // encrypt the block data using the block key
        let wrapped_data = ::sodiumoxide::crypto::secretbox::seal(self.data.as_slice(), &block_nonce, &block_key.as_sodium_secretbox_key());

        // wrap the block key with the main encryption key
        let wrapped_block_key = match block_key.to_wrapped(main, Some(&block_nonce)) {
            Ok(wk) => wk,
            Err(e) => return Err(e),
        };


        Ok(WrappedBlock { version: self.version, hmac: self.hmac, wrapped_data: wrapped_data, wrapped_key: wrapped_block_key, nonce: block_nonce })
    }
}

impl AsRef<[u8]> for Block {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}



pub struct WrappedBlock {
    version: SyncVersion,
    hmac: Vec<u8>,
    wrapped_data: Vec<u8>,
    nonce: ::sodiumoxide::crypto::secretbox::Nonce,
    wrapped_key: WrappedKey,
}


impl WrappedBlock {
    pub fn to_block(self, main: &Key) -> Result<Block, SDError> {

        let block_key = match self.wrapped_key.to_key(main, Some(&self.nonce)) {
            Ok(k) => k,
            Err(_) => return Err(SDError::CryptoError(CryptoError::BlockDecryptFailed))
        };

        let block_raw = match ::sodiumoxide::crypto::secretbox::open(&self.wrapped_data, &self.nonce, &block_key.as_sodium_secretbox_key()) {
            Ok(s) => s,
            Err(_) => return Err(SDError::CryptoError(CryptoError::BlockDecryptFailed))
        };


        Ok(Block { version: self.version, hmac: self.hmac, data: block_raw })
    }

    pub fn get_hmac(&self) -> Vec<u8> {
        self.hmac.clone()
    }

    pub fn to_binary(&self) -> Vec<u8> {
        let mut binary_data = Vec::new();

        // first 8 bytes are the file ID, version, and reserved area
        let magic: &'static [u8; 2] = br"sd";
        let file_type: &'static [u8; 1] = br"b";
        let version = self.version.as_ref();
        let reserved: &'static [u8; 3] = br"000";

        binary_data.extend(magic.as_ref());
        binary_data.extend(file_type.as_ref());
        binary_data.extend(version);
        binary_data.extend(reserved.as_ref());

        // next 48 bytes will be the wrapped block key
        binary_data.extend(self.wrapped_key.as_ref());

        // next 24 bytes will be the nonce
        let n: &[u8] = self.nonce.as_ref();
        binary_data.extend(n);
        assert!(binary_data.len() == magic.len() + file_type.len() + version.len() + reserved.len() + (SECRETBOX_KEY_SIZE + SECRETBOX_MAC_SIZE) + SECRETBOX_NONCE_SIZE);

        // remainder will be the encrypted block data
        binary_data.extend(self.wrapped_data.as_slice());

        binary_data
    }

    pub fn from(raw: Vec<u8>, hmac: Vec<u8>) -> Result<WrappedBlock, SDError> {

        let raw_block: BinaryFormat = match ::binformat::binary_parse(&raw) {
            Done(_, o) => o,
            Error(_) => return Err(SDError::BlockMissing),
            Incomplete(_) => panic!("should never happen")
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

        Ok(WrappedBlock { version: block_ver, hmac: hmac, wrapped_key: wrapped_block_key, wrapped_data: wrapped_block_raw, nonce: block_nonce })
    }

}



impl AsRef<[u8]> for WrappedBlock {
    fn as_ref(&self) -> &[u8] {
        self.wrapped_data.as_ref()
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