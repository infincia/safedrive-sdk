
// external crate imports

use ::bip39::{Bip39, Language};
use ::rustc_serialize::hex::{ToHex, FromHex};

// internal imports

use ::error::CryptoError;
use ::models::WrappedKeysetBody;
use ::constants::*;

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum KeyType {
    KeyTypeMaster,
    KeyTypeMain,
    KeyTypeHMAC,
    KeyTypeTweak,
    KeyTypeRecovery,
    KeyTypeSession,
    KeyTypeBlock,
}

impl KeyType {
    pub fn nonce(&self) -> ::sodiumoxide::crypto::secretbox::Nonce {
        let nonce_value = match *self {
            /// these are static for all accounts and must never change once in use
            ///
            /// they're static because they're only ever used for a single encryption operation
            /// which is inherently safe as far as nonce reuse is concerned
            KeyType::KeyTypeMaster => [1u8; 24],
            KeyType::KeyTypeMain => [2u8; 24],
            KeyType::KeyTypeHMAC => [3u8; 24],
            KeyType::KeyTypeTweak => [4u8; 24],
            _ => { panic!("other key types don't have a static nonce");  }
        };
        let nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_value).expect("failed to get nonce");

        nonce
    }
}

#[derive(Debug)]
pub struct WrappedKeyset {
    recovery: Option<String>,
    pub master: WrappedKey,
    pub main: WrappedKey,
    pub hmac: WrappedKey,
    pub tweak: WrappedKey
}

impl WrappedKeyset {
    pub fn new() -> Result<WrappedKeyset, CryptoError> {
        // generate a recovery phrase that will be used to encrypt the master key
        let mnemonic_keytype = ::bip39::KeyType::Key128;
        let mnemonic = try!(Bip39::new(&mnemonic_keytype, Language::English, ""));
        let recovery_phrase = { mnemonic.mnemonic.clone() };
        let recovery_key = Key::from(mnemonic);
        debug!("phrase: {}", recovery_phrase);

        // generate a master key and encrypt it with the recovery phrase and static nonce
        // We assign a specific, non-random nonce to use once for each key. Still safe, not reused.
        let master_key_type = KeyType::KeyTypeMaster;
        let master_key = Key::new(master_key_type);
        let master_key_wrapped = try!(master_key.to_wrapped(&recovery_key));

        // generate a main key and encrypt it with the master key and static nonce
        let main_key_type = KeyType::KeyTypeMain;
        let main_key = Key::new(main_key_type);
        let main_key_wrapped = try!(main_key.to_wrapped(&master_key));

        // generate an hmac key and encrypt it with the master key and static nonce
        let hmac_key_type = KeyType::KeyTypeHMAC;
        let hmac_key = Key::new(hmac_key_type);
        let hmac_key_wrapped = try!(hmac_key.to_wrapped(&master_key));

        // generate a tweak key and encrypt it with the master key and static nonce
        let tweak_key_type = KeyType::KeyTypeTweak;
        let tweak_key = Key::new(tweak_key_type);
        let tweak_key_wrapped = try!(tweak_key.to_wrapped(&master_key));
        debug!("generated key set");

        Ok( WrappedKeyset { recovery: Some(recovery_phrase), master: master_key_wrapped, main: main_key_wrapped, hmac: hmac_key_wrapped, tweak: tweak_key_wrapped })

    }

    pub fn to_keyset(&self, phrase: &str) -> Result<Keyset, CryptoError> {
        let mnemonic = try!(Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()));
        let recovery_key = Key::from(mnemonic);
        let master_key = try!(self.master.to_key(&recovery_key));
        let main_key = try!(self.main.to_key(&master_key));
        let hmac_key = try!(self.hmac.to_key(&master_key));
        let tweak_key = try!(self.tweak.to_key(&master_key));

        Ok(Keyset { recovery: phrase.to_string(), master: master_key, main: main_key, hmac: hmac_key, tweak: tweak_key })
    }

    pub fn recovery_phrase(&self) -> Option<String> {
        match self.recovery {
            Some(ref p) => Some(p.clone()),
            None => None
        }
    }

}

impl From<WrappedKeysetBody> for WrappedKeyset {
    fn from(body: WrappedKeysetBody) -> Self {
        let wrapped_master_key = WrappedKey::from_hex(body.master, KeyType::KeyTypeMaster).expect("failed to convert key hex to key");
        let wrapped_main_key = WrappedKey::from_hex(body.main, KeyType::KeyTypeMain).expect("failed to convert key hex to key");
        let wrapped_hmac_key = WrappedKey::from_hex(body.hmac, KeyType::KeyTypeHMAC).expect("failed to convert key hex to key");
        let wrapped_tweak_key = WrappedKey::from_hex(body.tweak, KeyType::KeyTypeTweak).expect("failed to convert key hex to key");

        WrappedKeyset { master: wrapped_master_key, main: wrapped_main_key, hmac: wrapped_hmac_key, tweak: wrapped_tweak_key, recovery: None }
    }

}


#[derive(Debug)]
pub struct Keyset {
    pub recovery: String,
    pub master: Key,
    pub main: Key,
    pub hmac: Key,
    pub tweak: Key
}

impl Keyset {

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

    pub fn to_key(&self, wrapping_key: &Key) -> Result<Key, CryptoError> {

        // decrypt the key with the recovery key and nonce
        let wrapping_key_s = ::sodiumoxide::crypto::secretbox::Key::from_slice(wrapping_key.as_ref()).expect("failed to get wrapping key struct");

        let key_raw = match ::sodiumoxide::crypto::secretbox::open(&self.bytes, &self.key_type.nonce(), &wrapping_key_s) {
            Ok(k) => k,
            Err(_) => return Err(CryptoError::RecoveryPhraseIncorrect)
        };

        Ok(Key { bytes: key_raw, key_type: self.key_type })
    }
}

impl AsRef<[u8]> for WrappedKey {
    fn as_ref(&self) -> &[u8] {
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

    fn new(key_type: KeyType) -> Key {
        let key_size = match key_type {
            KeyType::KeyTypeMaster => SECRETBOX_KEY_SIZE,
            KeyType::KeyTypeMain => SECRETBOX_KEY_SIZE,
            KeyType::KeyTypeHMAC => HMAC_KEY_SIZE,
            KeyType::KeyTypeTweak => SECRETBOX_KEY_SIZE,
            _ => { panic!("other key types don't have a static nonce");  }
        };
        let key = ::sodiumoxide::randombytes::randombytes(key_size);

        Key { bytes: key.to_vec(), key_type: key_type }
    }

    fn from(recovery_phrase: Bip39) -> Key {
        let recovery_key = ::sodiumoxide::crypto::hash::sha256::hash(recovery_phrase.seed.as_ref());
        Key { bytes: recovery_key.as_ref().to_vec(), key_type: KeyType::KeyTypeRecovery }
    }
}

pub trait KeyConversion {
    fn as_sodium_secretbox_key(&self) -> ::sodiumoxide::crypto::secretbox::Key;
    fn as_sodium_auth_key(&self) -> ::sodiumoxide::crypto::auth::Key;
    fn to_wrapped(&self, wrapping_key: &Key) -> Result<WrappedKey, CryptoError>;
}

impl KeyConversion for Key {

    fn as_sodium_secretbox_key(&self) -> ::sodiumoxide::crypto::secretbox::Key {
        match self.key_type {
            KeyType::KeyTypeMaster => {},
            KeyType::KeyTypeMain => {},
            KeyType::KeyTypeTweak => {},
            KeyType::KeyTypeRecovery => {},
            _ => { panic!("other key types don't have a static nonce");  }
        };
        ::sodiumoxide::crypto::secretbox::Key::from_slice(self.bytes.as_ref()).expect("failed to get secretbox key struct")
    }

    fn as_sodium_auth_key(&self) -> ::sodiumoxide::crypto::auth::Key {
        match self.key_type {
            KeyType::KeyTypeHMAC => HMAC_KEY_SIZE,
            _ => { panic!("other key types don't have a static nonce");  }
        };
        ::sodiumoxide::crypto::auth::Key::from_slice(self.bytes.as_ref()).expect("failed to get auth key struct")
    }

    fn to_wrapped(&self, wrapping_key: &Key) -> Result<WrappedKey, CryptoError> {
        let nonce = self.key_type.nonce();
        let wrapping_key_s = ::sodiumoxide::crypto::secretbox::Key::from_slice(wrapping_key.as_ref()).expect("failed to get wrapping key struct");
        let wrapped_key = ::sodiumoxide::crypto::secretbox::seal(self.bytes.as_ref(), &nonce, &wrapping_key_s);


        Ok(WrappedKey { bytes: wrapped_key, key_type: self.key_type })
    }
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

#[test]
fn key_generate_test() {
    let wrapped_keyset = match WrappedKeyset::new() {
        Ok(wks) => wks,
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn key_unwrap_test() {
    let wrapped_keyset = match WrappedKeyset::new() {
        Ok(wks) => wks,
        Err(_) => { assert!(true == false); return }
    };

    let phrase = wrapped_keyset.recovery_phrase().unwrap();

    match wrapped_keyset.to_keyset(&phrase) {
        Ok(_) => {},
        Err(_) => { assert!(true == false); return }
    };
}
