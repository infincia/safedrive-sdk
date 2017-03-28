
/// external crate imports

use ::bip39::{Bip39, Language};
use ::rustc_serialize::hex::{ToHex, FromHex};

/// internal imports

use ::error::CryptoError;
use ::models::WrappedKeysetBody;
use ::constants::*;

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum KeyType {
    Master,
    Main,
    HMAC,
    Tweak,
    Recovery,
    Session,
    Block,
}

impl KeyType {
    pub fn key_wrapping_nonce(&self) -> ::sodiumoxide::crypto::secretbox::Nonce {
        let nonce_value = match *self {
            /// these are static for all accounts and must never change once in use
            ///
            /// they're static because they're only ever used for a single encryption operation
            /// which is inherently safe as far as nonce reuse is concerned
            KeyType::Master => [1u8; 24],
            KeyType::Main => [2u8; 24],
            KeyType::HMAC => [3u8; 24],
            KeyType::Tweak => [4u8; 24],
            _ => { panic!("other key types don't have a static nonce");  }
        };
        let nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_value).expect("failed to get nonce");

        nonce
    }
}

impl<'a> ::std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {
            KeyType::Master => {
                write!(f, "KeyType<Master>")
            },
            KeyType::Main => {
                write!(f, "KeyType<Main>")
            },
            KeyType::HMAC => {
                write!(f, "KeyType<HMAC>")
            },
            KeyType::Tweak => {
                write!(f, "KeyType<Tweak>")
            },
            KeyType::Recovery => {
                write!(f, "KeyType<Recovery>")
            },
            KeyType::Session => {
                write!(f, "KeyType<Session>")
            },
            KeyType::Block => {
                write!(f, "KeyType<Block>")
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct WrappedKeyset {
    recovery: Option<String>,
    pub master: WrappedKey,
    pub main: WrappedKey,
    pub hmac: WrappedKey,
    pub tweak: WrappedKey
}

impl WrappedKeyset {
    pub fn new() -> Result<WrappedKeyset, CryptoError> {
        /// generate a recovery phrase that will be used to encrypt the master key
        let mnemonic_keytype = ::bip39::KeyType::Key128;
        let mnemonic = try!(Bip39::new(&mnemonic_keytype, Language::English, ""));
        let recovery_phrase = { mnemonic.mnemonic.clone() };
        let recovery_key = Key::from(mnemonic);
        debug!("phrase: {}", recovery_phrase);

        /// generate a master key and encrypt it with the recovery phrase and static nonce
        /// We assign a specific, non-random nonce to use once for each key. Still safe, not reused.
        let master_key_type = KeyType::Master;
        let master_key = Key::new(master_key_type);
        let master_key_wrapped = try!(master_key.to_wrapped(&recovery_key, None));

        /// generate a main key and encrypt it with the master key and static nonce
        let main_key_type = KeyType::Main;
        let main_key = Key::new(main_key_type);
        let main_key_wrapped = try!(main_key.to_wrapped(&master_key, None));

        /// generate an hmac key and encrypt it with the master key and static nonce
        let hmac_key_type = KeyType::HMAC;
        let hmac_key = Key::new(hmac_key_type);
        let hmac_key_wrapped = try!(hmac_key.to_wrapped(&master_key, None));

        /// generate a tweak key and encrypt it with the master key and static nonce
        let tweak_key_type = KeyType::Tweak;
        let tweak_key = Key::new(tweak_key_type);
        let tweak_key_wrapped = try!(tweak_key.to_wrapped(&master_key, None));
        debug!("generated key set");

        Ok( WrappedKeyset { recovery: Some(recovery_phrase), master: master_key_wrapped, main: main_key_wrapped, hmac: hmac_key_wrapped, tweak: tweak_key_wrapped })

    }

    pub fn to_keyset(&self, phrase: &str) -> Result<Keyset, CryptoError> {
        let mnemonic = try!(Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()));
        let recovery_key = Key::from(mnemonic);
        let master_key = try!(self.master.to_key(&recovery_key, None));
        let main_key = try!(self.main.to_key(&master_key, None));
        let hmac_key = try!(self.hmac.to_key(&master_key, None));
        let tweak_key = try!(self.tweak.to_key(&master_key, None));

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
        let wrapped_master_key = WrappedKey::from_hex(body.master, KeyType::Master).expect("failed to convert key hex to key");
        let wrapped_main_key = WrappedKey::from_hex(body.main, KeyType::Main).expect("failed to convert key hex to key");
        let wrapped_hmac_key = WrappedKey::from_hex(body.hmac, KeyType::HMAC).expect("failed to convert key hex to key");
        let wrapped_tweak_key = WrappedKey::from_hex(body.tweak, KeyType::Tweak).expect("failed to convert key hex to key");

        WrappedKeyset { master: wrapped_master_key, main: wrapped_main_key, hmac: wrapped_hmac_key, tweak: wrapped_tweak_key, recovery: None }
    }

}

impl<'a> ::std::fmt::Display for WrappedKeyset {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "WrappedKeyset <master:{}, main:{}, hmac:{}, tweak:{}>", self.master, self.main, self.hmac, self.tweak)
    }
}

#[derive(Debug, Clone)]
pub struct Keyset {
    pub recovery: String,
    pub master: Key,
    pub main: Key,
    pub hmac: Key,
    pub tweak: Key
}

impl Keyset {

}



#[derive(Debug, Clone)]
pub struct WrappedKey {
    bytes: Vec<u8>,
    key_type: KeyType
}

impl WrappedKey {
    pub fn from(key: Vec<u8>, key_type: KeyType) -> WrappedKey {

        WrappedKey { bytes: key, key_type: key_type }
    }

    pub fn from_hex(hex_key: String, key_type: KeyType) -> Result<WrappedKey, CryptoError> {
        Ok(WrappedKey::from(try!(hex_key.from_hex()), key_type))
    }

    pub fn to_key(&self, wrapping_key: &Key, nonce: Option<&::sodiumoxide::crypto::secretbox::Nonce>) -> Result<Key, CryptoError> {

        let n = match self.key_type {
            KeyType::Master |
            KeyType::Main |
            KeyType::HMAC |
            KeyType::Tweak => {
                /// all use a static nonce when wrapping their key type, MUST NOT use a random nonce
                self.key_type.key_wrapping_nonce()
            },
            KeyType::Block | KeyType::Session => {
                /// use a non-static nonce when wrapping their key type, MUST NOT use a static nonce
                nonce.expect("attempted to unwrap a block or session key without an external random nonce").clone()
            },
            _ => { panic!("other key types cannot be wrapped");  }
        };

        /// decrypt the key with the wrapping key and nonce
        let wrapping_key_s = wrapping_key.as_sodium_secretbox_key();

        let key_raw = match ::sodiumoxide::crypto::secretbox::open(&self.bytes, &n, &wrapping_key_s) {
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



impl<'a> ::std::fmt::Display for WrappedKey {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "WrappedKey <length:{}: {:?}>", self.bytes.len(), self.bytes)
    }
}


#[derive(Debug, Clone)]
pub struct Key {
    bytes: Vec<u8>,
    key_type: KeyType
}

impl Key {

    pub fn new(key_type: KeyType) -> Key {
        let key_size = match key_type {
            KeyType::Master => SECRETBOX_KEY_SIZE,
            KeyType::Main => SECRETBOX_KEY_SIZE,
            KeyType::HMAC => HMAC_KEY_SIZE,
            KeyType::Tweak => SECRETBOX_KEY_SIZE,
            KeyType::Block => SECRETBOX_KEY_SIZE,
            KeyType::Session => SECRETBOX_KEY_SIZE,
            _ => { panic!("other key types can't be created this way");  }
        };
        let key = ::sodiumoxide::randombytes::randombytes(key_size);

        Key { bytes: key.to_vec(), key_type: key_type }
    }

    fn from(recovery_phrase: Bip39) -> Key {
        let recovery_key = ::sodiumoxide::crypto::hash::sha256::hash(recovery_phrase.seed.as_ref());
        Key { bytes: recovery_key.as_ref().to_vec(), key_type: KeyType::Recovery }
    }

    pub fn as_sodium_secretbox_key(&self) -> ::sodiumoxide::crypto::secretbox::Key {
        match self.key_type {
            KeyType::Master => {},
            KeyType::Main => {},
            KeyType::Tweak => {},
            KeyType::Recovery => {},
            KeyType::Block => {},
            KeyType::Session => {},
            _ => { panic!("other key types can't be used as a secretbox key");  }
        };
        ::sodiumoxide::crypto::secretbox::Key::from_slice(self.bytes.as_ref()).expect("failed to get secretbox key struct")
    }

    pub fn as_sodium_auth_key(&self) -> ::sodiumoxide::crypto::auth::Key {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => { panic!("other key types can't be used as an auth key");  }
        };
        ::sodiumoxide::crypto::auth::Key::from_slice(self.bytes.as_ref()).expect("failed to get auth key struct")
    }

    pub fn as_blake2_64(&self) -> &[u8] {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => { panic!("other key types can't be used as a blake2 key");  }
        };
        let s = self.bytes.as_slice();
        let k = &s[0..8];

        k
    }

    pub fn as_blake2_128(&self) -> &[u8] {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => { panic!("other key types can't be used as a blake2 key");  }
        };
        let s = self.bytes.as_slice();
        let k = &s[0..16];

        k
    }

    pub fn as_blake2_192(&self) -> &[u8] {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => { panic!("other key types can't be used as a blake2 key");  }
        };
        let s = self.bytes.as_slice();
        let k = &s[0..24];

        k
    }

    pub fn as_blake2_256(&self) -> &[u8] {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => { panic!("other key types can't be used as a blake2 key");  }
        };
        let s = self.bytes.as_slice();
        let k = &s[0..32];

        k
    }

    pub fn to_wrapped(&self, wrapping_key: &Key, nonce: Option<&::sodiumoxide::crypto::secretbox::Nonce>) -> Result<WrappedKey, CryptoError> {
        let n = match self.key_type {
            KeyType::Master |
            KeyType::Main |
            KeyType::HMAC |
            KeyType::Tweak => {
                /// use a static nonce when wrapping this key type, MUST NOT use a random nonce
                self.key_type.key_wrapping_nonce()
            },
            KeyType::Block | KeyType::Session => {
                /// use a random nonce when wrapping this key type, MUST NOT use a static nonce
                nonce.expect("attempted to wrap a block or session key without an external random nonce").clone()
            },
            _ => { panic!("other key types cannot be wrapped");  }
        };

        let wrapping_key_s = wrapping_key.as_sodium_secretbox_key();

        let wrapped_key = ::sodiumoxide::crypto::secretbox::seal(self.bytes.as_ref(), &n, &wrapping_key_s);

        Ok(WrappedKey::from(wrapped_key, self.key_type))
    }
}


#[test]
fn key_generate_test() {
    let _ = match WrappedKeyset::new() {
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


#[test]
fn key_unwrap_lowercase_master_generated_by_sdk_00318_test() {
    let phrase = "plastic quote hotel coyote exercise stairs runway protect visit lion arch truth";

    let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()).expect("failed to parse bip39 phrase");
    let recovery_key = Key::from(mnemonic);

    let wrapped_master_hex = "f4814a238215f659585052a6d21cba789e60c3fdcb7ae095641366857aae59f4a374b564e24d41a1619ec13118873621";

    let wrapped_master = WrappedKey::from_hex(wrapped_master_hex.to_owned(), KeyType::Master).expect("Failed to decode key hex");
    match wrapped_master.to_key(&recovery_key, None) {
        Ok(_) => {},
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return
        }
    };
}

#[test]
fn key_unwrap_uppercase_master_generated_by_sdk_00318_test() {
    let phrase = "plastic quote hotel coyote exercise stairs runway protect visit lion arch truth";

    let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()).expect("failed to parse bip39 phrase");
    let recovery_key = Key::from(mnemonic);

    let wrapped_master_hex = "F4814A238215F659585052A6D21CBA789E60C3FDCB7AE095641366857AAE59F4A374B564E24D41A1619EC13118873621";

    let wrapped_master = WrappedKey::from_hex(wrapped_master_hex.to_owned(), KeyType::Master).expect("Failed to decode key hex");
    match wrapped_master.to_key(&recovery_key, None) {
        Ok(_) => {},
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return
        }
    };
}

