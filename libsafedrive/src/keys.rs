extern crate sodiumoxide;

extern crate rustc_serialize;

use self::rustc_serialize::hex::{ToHex, FromHex};

use error::CryptoError;


#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum KeyType {
    KeyTypeMaster,
    KeyTypeMain,
    KeyTypeHMAC,
    KeyTypeTweak,
    KeyTypeRecovery
}

impl KeyType {
    pub fn nonce(&self) -> sodiumoxide::crypto::secretbox::Nonce {
        let nonce_value = match *self {
            /// these are static for all accounts and must never change once in use
            ///
            /// they're static because they're only ever used for a single encryption operation
            /// which is inherently safe as far as nonce reuse is concerned
            KeyType::KeyTypeMaster => [1u8; 24],
            KeyType::KeyTypeMain => [2u8; 24],
            KeyType::KeyTypeHMAC => [3u8; 24],
            KeyType::KeyTypeTweak => [4u8; 24],
            KeyType::KeyTypeRecovery => { panic!("recovery key has no nonce, it is not wrapped!");  }
        };
        let nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_value).expect("failed to get nonce");

        nonce
    }
}


#[derive(Debug)]
pub struct WrappedKey {
    pub bytes: Vec<u8>,
    pub key_type: KeyType
}

impl WrappedKey {
    pub fn new(key: Vec<u8>, key_type: KeyType) -> WrappedKey {

        WrappedKey { bytes: key, key_type: key_type }
    }

    pub fn from_hex(hex_key: String, key_type: KeyType) -> Result<WrappedKey, CryptoError> {
        Ok(WrappedKey::new(try!(hex_key.from_hex()), key_type))
    }

    pub fn to_key(&self, wrapping_key: &[u8]) -> Result<Key, CryptoError> {

        // decrypt the key with the recovery key and nonce
        let wrapping_key_s = sodiumoxide::crypto::secretbox::Key::from_slice(wrapping_key).expect("failed to get wrapping key struct");

        let key_raw = match sodiumoxide::crypto::secretbox::open(&self.bytes, &self.key_type.nonce(), &wrapping_key_s) {
            Ok(k) => k,
            Err(_) => return Err(CryptoError::DecryptFailed)
        };

        Ok(Key { bytes: key_raw, key_type: self.key_type })
    }

    pub fn as_ref(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl ToHex for WrappedKey {
    fn to_hex(&self) -> String {
        self.bytes.to_hex()
    }
}

#[derive(Debug)]
pub struct Key {
    pub bytes: Vec<u8>,
    pub key_type: KeyType
}

impl Key {

    pub fn new(key_type: KeyType) -> Key {
        let key_size = sodiumoxide::crypto::secretbox::KEYBYTES;
        let key = sodiumoxide::randombytes::randombytes(key_size);

        Key { bytes: key.to_vec(), key_type: key_type }
    }

    pub fn as_sodium_key(&self) -> sodiumoxide::crypto::secretbox::Key {
        sodiumoxide::crypto::secretbox::Key::from_slice(self.bytes.as_ref()).expect("failed to get key struct")
    }

    pub fn to_wrapped(&self, wrapping_key: &[u8]) -> Result<WrappedKey, CryptoError> {
        let nonce = self.key_type.nonce();
        let wrapping_key_s = sodiumoxide::crypto::secretbox::Key::from_slice(wrapping_key).expect("failed to get wrapping key struct");
        let wrapped_key = sodiumoxide::crypto::secretbox::seal(self.bytes.as_ref(), &nonce, &wrapping_key_s);


        Ok(WrappedKey { bytes: wrapped_key, key_type: self.key_type })
    }

    pub fn as_ref(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}
