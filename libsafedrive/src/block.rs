
use ::models::{SyncVersion};

use ::error::{CryptoError, SDError};
use ::binformat::BinaryFormat;
use ::nom::IResult::*;
use ::keys::{Key, WrappedKey, KeyType, KeyConversion};
use ::rustc_serialize::hex::{ToHex};

use ::constants::*;


#[derive(Debug, Clone)]
pub struct Block {
    version: SyncVersion,
    pub hmac: Vec<u8>,
    pub data: Vec<u8>,
}

impl Block {
    pub fn new(version: SyncVersion, hmac: &Key, data: Vec<u8>) -> Block {
        // calculate hmac of the block
        let hmac_key = hmac.as_sodium_auth_key();

        let block_hmac = {
            let raw_chunk = data.as_slice();

            let tag = ::sodiumoxide::crypto::auth::authenticate(raw_chunk, &hmac_key);

            let bh = tag.as_ref().to_vec();

            bh
        };
        Block { version: version, data: data, hmac: block_hmac }
    }

    pub fn name(&self) -> String {
        self.hmac.to_hex()
    }

    pub fn get_hmac(&self) -> Vec<u8> {
        self.hmac.clone()
    }

    pub fn to_wrapped(self, main: &Key) -> Result<WrappedBlock, CryptoError> {

        // generate a new block key
        let block_key_raw = ::sodiumoxide::randombytes::randombytes(SECRETBOX_KEY_SIZE);
        let block_key_struct = ::sodiumoxide::crypto::secretbox::Key::from_slice(&block_key_raw)
            .expect("failed to get block key struct");

        // We use the first 24 bytes of the block hmac value as nonce for wrapping
        // the block key and encrypting the block itself.
        //
        // This is cryptographically safe but still deterministic: encrypting
        // the same block twice with a specific key will always produce the same
        // output block, which is critical for versioning and deduplication
        // across all backups of all sync folders
        let nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&self.hmac.as_slice()[0..SECRETBOX_NONCE_SIZE as usize])
            .expect("failed to get nonce");

        // we use the same nonce both while wrapping the block key, and the block data itself
        // this is safe because using the same nonce with 2 different keys is not nonce reuse

        // encrypt the block data using the block key
        let wrapped_data = ::sodiumoxide::crypto::secretbox::seal(self.data.as_slice(), &nonce, &block_key_struct);

        // wrap the block key with the main encryption key
        let wrapped_block_key_raw = ::sodiumoxide::crypto::secretbox::seal(&block_key_raw, &nonce, &(main.as_sodium_secretbox_key()));
        assert!(wrapped_block_key_raw.len() == SECRETBOX_KEY_SIZE + SECRETBOX_MAC_SIZE);
        let wrapped_block_key = WrappedKey::new(wrapped_block_key_raw, KeyType::Session);


        Ok(WrappedBlock { version: self.version, hmac: self.hmac, wrapped_data: wrapped_data, wrapped_key: wrapped_block_key })
    }
}




pub struct WrappedBlock {
    pub version: SyncVersion,
    pub hmac: Vec<u8>,
    pub wrapped_data: Vec<u8>,
    wrapped_key: WrappedKey,
}


impl WrappedBlock {
    pub fn to_block(self, main: &Key) -> Result<Block, SDError> {
        let nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&self.hmac)
            .expect("failed to get nonce");

        let main_key_s = ::sodiumoxide::crypto::secretbox::Key::from_slice(main.as_ref())
            .expect("failed to get main key struct");

        let block_key_raw = match ::sodiumoxide::crypto::secretbox::open(self.wrapped_key.as_ref(), &nonce, &main_key_s) {
            Ok(k) => k,
            Err(_) => return Err(SDError::CryptoError(CryptoError::BlockDecryptFailed))
        };

        let block_key_s = ::sodiumoxide::crypto::secretbox::Key::from_slice(&block_key_raw).expect("failed to get unwrapped block key struct");

        let block_raw = match ::sodiumoxide::crypto::secretbox::open(&self.wrapped_data, &nonce, &block_key_s) {
            Ok(s) => s,
            Err(_) => return Err(SDError::CryptoError(CryptoError::BlockDecryptFailed))
        };


        Ok(Block { version: self.version, hmac: self.hmac, data: block_raw })
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

        // next 24 bytes will be the nonce/hmac
        binary_data.extend(self.hmac.as_slice());
        assert!(binary_data.len() == magic.len() + file_type.len() + version.len() + reserved.len() + (SECRETBOX_KEY_SIZE + SECRETBOX_MAC_SIZE) + SECRETBOX_NONCE_SIZE);

        // remainder will be the encrypted block data90
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
            "00" => SyncVersion::Version0,
            "01" => SyncVersion::Version1,
            "02" => SyncVersion::Version2,
            _ => panic!("unknown binary version")
        };
        let wrapped_block_key_raw = raw_block.wrapped_key.to_vec();
        // let nonce_raw = raw_block.nonce.to_vec();
        let wrapped_block_raw = raw_block.wrapped_data.to_vec();

        let wrapped_block_key = WrappedKey::from(wrapped_block_key_raw, KeyType::Block);

        Ok(WrappedBlock { version: block_ver, hmac: hmac, wrapped_key: wrapped_block_key, wrapped_data: wrapped_block_raw })
    }

}