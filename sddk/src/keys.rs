use std::cmp::PartialEq;

/// external crate imports

use bip39::{Bip39, Language};
use rustc_serialize::hex::{ToHex, FromHex};
use reed_solomon::Encoder as RSEncoder;
use reed_solomon::Decoder as RSDecoder;
use reed_solomon::Buffer as RSBuffer;

use serde::{Serialize, Deserialize, Serializer, Deserializer};

/// internal imports

use error::CryptoError;
use models::WrappedKeysetBody;
use constants::*;

#[derive(Serialize, Deserialize)]
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
            _ => {
                panic!("other key types don't have a static nonce");
            },
        };
        let nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_value).expect("failed to get nonce");

        nonce
    }
}

impl<'a> ::std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {
            KeyType::Master => write!(f, "KeyType<Master>"),
            KeyType::Main => write!(f, "KeyType<Main>"),
            KeyType::HMAC => write!(f, "KeyType<HMAC>"),
            KeyType::Tweak => write!(f, "KeyType<Tweak>"),
            KeyType::Recovery => write!(f, "KeyType<Recovery>"),
            KeyType::Session => write!(f, "KeyType<Session>"),
            KeyType::Block => write!(f, "KeyType<Block>"),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct WrappedKeyset {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    recovery: Option<String>,
    pub master: WrappedKey,
    pub main: WrappedKey,
    pub hmac: WrappedKey,
    pub tweak: WrappedKey,
}

impl WrappedKeyset {
    pub fn new() -> Result<WrappedKeyset, CryptoError> {
        /// generate a recovery phrase that will be used to encrypt the master key
        let mnemonic_keytype = ::bip39::KeyType::Key128;
        let mnemonic = Bip39::new(&mnemonic_keytype, Language::English, "")?;
        let recovery_phrase = { mnemonic.mnemonic.clone() };
        let recovery_key = Key::from(mnemonic);

        /// generate a master key and encrypt it with the recovery phrase and static nonce
        /// We assign a specific, non-random nonce to use once for each key. Still safe, not reused.
        let master_key_type = KeyType::Master;
        let master_key = Key::new(master_key_type);
        let master_key_wrapped = master_key.to_wrapped(&recovery_key, None)?;

        /// generate a main key and encrypt it with the master key and static nonce
        let main_key_type = KeyType::Main;
        let main_key = Key::new(main_key_type);
        let main_key_wrapped = main_key.to_wrapped(&master_key, None)?;

        /// generate an hmac key and encrypt it with the master key and static nonce
        let hmac_key_type = KeyType::HMAC;
        let hmac_key = Key::new(hmac_key_type);
        let hmac_key_wrapped = hmac_key.to_wrapped(&master_key, None)?;

        /// generate a tweak key and encrypt it with the master key and static nonce
        let tweak_key_type = KeyType::Tweak;
        let tweak_key = Key::new(tweak_key_type);
        let tweak_key_wrapped = tweak_key.to_wrapped(&master_key, None)?;
        info!("new recovery phrase: {}", recovery_phrase);
        info!("new master key: {}", master_key_wrapped.to_hex());
        info!("new main key: {}", main_key_wrapped.to_hex());
        info!("new hmac key: {}", hmac_key_wrapped.to_hex());
        info!("new tweak key: {}", tweak_key_wrapped.to_hex());


        Ok(WrappedKeyset {
               recovery: Some(recovery_phrase),
               master: master_key_wrapped,
               main: main_key_wrapped,
               hmac: hmac_key_wrapped,
               tweak: tweak_key_wrapped,
           })

    }

    pub fn to_keyset(&self, phrase: &str) -> Result<Keyset, CryptoError> {
        let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string())?;
        let recovery_key = Key::from(mnemonic);
        let master_key = self.master.to_key(&recovery_key, None)?;
        let main_key = self.main.to_key(&master_key, None)?;
        let hmac_key = self.hmac.to_key(&master_key, None)?;
        let tweak_key = self.tweak.to_key(&master_key, None)?;

        Ok(Keyset {
               recovery: phrase.to_string(),
               master: master_key,
               main: main_key,
               hmac: hmac_key,
               tweak: tweak_key,
               has_ecc: self.master.has_ecc && self.main.has_ecc && self.hmac.has_ecc && self.tweak.has_ecc,
           })
    }

    pub fn recovery_phrase(&self) -> Option<String> {
        match self.recovery {
            Some(ref p) => Some(p.clone()),
            None => None,
        }
    }
}

impl From<WrappedKeysetBody> for WrappedKeyset {
    fn from(body: WrappedKeysetBody) -> Self {
        let wrapped_master_key = WrappedKey::from_hex(body.master, KeyType::Master).expect("failed to convert key hex to key");
        let wrapped_main_key = WrappedKey::from_hex(body.main, KeyType::Main).expect("failed to convert key hex to key");
        let wrapped_hmac_key = WrappedKey::from_hex(body.hmac, KeyType::HMAC).expect("failed to convert key hex to key");
        let wrapped_tweak_key = WrappedKey::from_hex(body.tweak, KeyType::Tweak).expect("failed to convert key hex to key");

        WrappedKeyset {
            master: wrapped_master_key,
            main: wrapped_main_key,
            hmac: wrapped_hmac_key,
            tweak: wrapped_tweak_key,
            recovery: None,
        }
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
    pub tweak: Key,
    pub has_ecc: bool,
}

impl Keyset {

}


#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct WrappedKey {
    bytes: Vec<u8>,
    key_type: KeyType,
    has_ecc: bool,
}

impl WrappedKey {
    pub fn from(key: Vec<u8>, key_type: KeyType) -> WrappedKey {

        WrappedKey {
            bytes: key,
            key_type: key_type,
            has_ecc: false,
        }
    }

    pub fn from_rs(coded_key: Vec<u8>, key_type: KeyType) -> Result<WrappedKey, CryptoError> {
        let mut key = coded_key;

        let dec = RSDecoder::new(KEY_ECC_LEN);

        let recovered: RSBuffer = dec.correct(&mut key, None)?;

        let recovered_data = recovered.data();
        let original_data = &key[..recovered.data().len()];
        if &recovered_data != &original_data {
            warn!("KEY CORRUPTION DETECTED: {}", key_type);
            warn!("KEY CORRUPTION ORIGINAL: {:?}", original_data);
            warn!("KEY CORRUPTION RECOVERED: {:?}", recovered_data);
        }

        Ok(WrappedKey::from(recovered.data().to_vec(), key_type))
    }

    pub fn from_hex(hex_key: String, key_type: KeyType) -> Result<WrappedKey, CryptoError> {
        let key = hex_key.from_hex()?;

        // short path if the key has no ecc info
        if key.len() == 48 {
            warn!("Key type {} is a legacy key", key_type);
            return Ok(WrappedKey::from(key, key_type));
        } else {
            info!("Key type {} is a modern ECC key", key_type);
        }

        let mut recovered = WrappedKey::from_rs(key, key_type)?;
        recovered.has_ecc = true;

        Ok(recovered)
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
            _ => {
                panic!("other key types cannot be wrapped");
            },
        };

        /// decrypt the key with the wrapping key and nonce
        let wrapping_key_s = wrapping_key.as_sodium_secretbox_key();

        let key_raw = match ::sodiumoxide::crypto::secretbox::open(&self.bytes, &n, &wrapping_key_s) {
            Ok(k) => k,
            Err(_) => return Err(CryptoError::RecoveryPhraseIncorrect),
        };

        Ok(Key {
               bytes: key_raw,
               key_type: self.key_type,
           })
    }

    fn to_rs(&self) -> Vec<u8> {
        let enc = RSEncoder::new(KEY_ECC_LEN);
        let encoded = enc.encode(self.bytes.as_slice());

        let ecc = *encoded;

        ecc.to_vec()
    }
}

impl AsRef<[u8]> for WrappedKey {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl ToHex for WrappedKey {
    fn to_hex(&self) -> String {
        self.to_rs().to_hex()
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
    key_type: KeyType,
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
            _ => {
                panic!("other key types can't be created this way");
            },
        };
        let key = ::sodiumoxide::randombytes::randombytes(key_size);

        Key {
            bytes: key.to_vec(),
            key_type: key_type,
        }
    }

    fn from(recovery_phrase: Bip39) -> Key {
        let recovery_key = ::sodiumoxide::crypto::hash::sha256::hash(recovery_phrase.seed.as_ref());
        Key {
            bytes: recovery_key.as_ref().to_vec(),
            key_type: KeyType::Recovery,
        }
    }

    pub fn as_sodium_secretbox_key(&self) -> ::sodiumoxide::crypto::secretbox::Key {
        match self.key_type {
            KeyType::Master => {},
            KeyType::Main => {},
            KeyType::Tweak => {},
            KeyType::Recovery => {},
            KeyType::Block => {},
            KeyType::Session => {},
            _ => {
                panic!("other key types can't be used as a secretbox key");
            },
        };
        ::sodiumoxide::crypto::secretbox::Key::from_slice(self.bytes.as_ref()).expect("failed to get secretbox key struct")
    }

    pub fn as_sodium_auth_key(&self) -> ::sodiumoxide::crypto::auth::Key {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => {
                panic!("other key types can't be used as an auth key");
            },
        };
        ::sodiumoxide::crypto::auth::Key::from_slice(self.bytes.as_ref()).expect("failed to get auth key struct")
    }

    pub fn as_blake2_64(&self) -> &[u8] {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => {
                panic!("other key types can't be used as a blake2 key");
            },
        };
        let s = self.bytes.as_slice();
        let k = &s[0..8];

        k
    }

    pub fn as_blake2_128(&self) -> &[u8] {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => {
                panic!("other key types can't be used as a blake2 key");
            },
        };
        let s = self.bytes.as_slice();
        let k = &s[0..16];

        k
    }

    pub fn as_blake2_192(&self) -> &[u8] {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => {
                panic!("other key types can't be used as a blake2 key");
            },
        };
        let s = self.bytes.as_slice();
        let k = &s[0..24];

        k
    }

    pub fn as_blake2_256(&self) -> &[u8] {
        match self.key_type {
            KeyType::HMAC => {},
            KeyType::Tweak => {},
            _ => {
                panic!("other key types can't be used as a blake2 key");
            },
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
            _ => {
                panic!("other key types cannot be wrapped");
            },
        };

        let wrapping_key_s = wrapping_key.as_sodium_secretbox_key();

        let wrapped_key = ::sodiumoxide::crypto::secretbox::seal(self.bytes.as_ref(), &n, &wrapping_key_s);

        Ok(WrappedKey::from(wrapped_key, self.key_type))
    }
}


#[test]
fn keyset_generate_test() {
    let _ = match WrappedKeyset::new() {
        Ok(wks) => wks,
        Err(_) => {
            assert!(true == false);
            return;
        },
    };
}

#[test]
fn keyset_backup_test() {
    let wks = match WrappedKeyset::new() {
        Ok(wks) => wks,
        Err(_) => {
            assert!(true == false);
            return;
        },
    };

    let cargo_path: &str = env!("CARGO_MANIFEST_DIR");


    let mut p: ::std::path::PathBuf = ::std::path::PathBuf::from(cargo_path);
    p.push("target");

    match ::util::write_backup_keyset(Some(p), &wks, "user@safedrive.io") {
        Ok(()) => {

        },
        Err(err) => {
            panic!("{}", err);
        }
    }
}

#[test]
fn key_generate_test() {
    let key = Key::new(KeyType::Master);

    assert!(key.bytes.len() == 32, "key size invalid");
}

#[test]
fn key_wrap_test() {
    let master_key = Key::new(KeyType::Master);
    let main_key = Key::new(KeyType::Main);
    let wrapped_main_key: WrappedKey = main_key.to_wrapped(&master_key, None).expect("failed to wrap key");
    assert!(wrapped_main_key.bytes.len() == 48, "invalid key length");
}

#[test]
fn key_rs_correction_test() {
    let master_key = Key::new(KeyType::Master);
    let main_key = Key::new(KeyType::Main);

    let wrapped_main_key: WrappedKey = main_key.to_wrapped(&master_key, None).expect("failed to wrap key");
    assert!(wrapped_main_key.bytes.len() == 48, "invalid key length");

    let mut wrapped_main_key_ecc = wrapped_main_key.to_rs();
    assert!(wrapped_main_key_ecc.len() == 96, "invalid ecc key length");
    let mut wrapped_main_key_ecc_slice = wrapped_main_key_ecc.as_mut_slice();

    // destroy 12 bytes of the key in different places
    wrapped_main_key_ecc_slice[0] = 0;
    wrapped_main_key_ecc_slice[2] = 0;
    wrapped_main_key_ecc_slice[3] = 0;
    wrapped_main_key_ecc_slice[4] = 0;
    wrapped_main_key_ecc_slice[5] = 0;
    wrapped_main_key_ecc_slice[6] = 0;
    wrapped_main_key_ecc_slice[7] = 0;
    wrapped_main_key_ecc_slice[12] = 0;
    wrapped_main_key_ecc_slice[14] = 0;
    wrapped_main_key_ecc_slice[20] = 0;
    wrapped_main_key_ecc_slice[24] = 0;
    wrapped_main_key_ecc_slice[30] = 0;

    // destroy part of the ecc code
    wrapped_main_key_ecc_slice[95] = 0;

    assert!(wrapped_main_key_ecc_slice.len() == 96, "invalid ecc key length");


    let wrapped_main_key_corrupted_hex = wrapped_main_key_ecc_slice.to_hex();
    assert!(wrapped_main_key_corrupted_hex.len() == 192, "invalid ecc hex key length");


    let recovered_main_key = WrappedKey::from_hex(wrapped_main_key_corrupted_hex, KeyType::Main).expect("key recovery failed");

    let main = recovered_main_key.to_key(&master_key, None).expect("failed to unwrap recovered key");
    assert!(main.bytes.len() == 32, "invalid key length");

}

#[test]
fn key_unwrap_test() {
    let wrapped_keyset = match WrappedKeyset::new() {
        Ok(wks) => wks,
        Err(_) => {
            assert!(true == false);
            return;
        },
    };

    let phrase = wrapped_keyset.recovery_phrase().unwrap();

    match wrapped_keyset.to_keyset(&phrase) {
        Ok(_) => {},
        Err(_) => {
            assert!(true == false);
            return;
        },
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
            return;
        },
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
            return;
        },
    };
}

#[test]
fn key_unwrap_uppercase_master_generated_by_sdk_00600_test() {
    let phrase = "special cheap live sing proud ethics public you apology outside empty person";

    let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()).expect("failed to parse bip39 phrase");
    let recovery_key = Key::from(mnemonic);

    let wrapped_master_hex = "CAEC81365F37D055C53545E74C34A35C8031D6E6810BFEEBBC119C7C34892176BBF19AE24C322EF6F9B385B1C5B01640C0CFFAA2AA4827DF9E054E12E73251C2F7D7713C778A2D14BACAF587B01EF2BD51D3A883D4CBA0C6DC0E6E702DB7568F";

    let wrapped_master = WrappedKey::from_hex(wrapped_master_hex.to_owned(), KeyType::Master).expect("Failed to decode key hex");
    assert!(wrapped_master.bytes.len() == 48, "invalid wrapped master key length");

    let master = match wrapped_master.to_key(&recovery_key, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };

    assert!(master.bytes.len() == 32, "invalid master key length");
}

#[test]
fn key_unwrap_lowercase_master_generated_by_sdk_00600_test() {
    let phrase = "special cheap live sing proud ethics public you apology outside empty person";

    let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()).expect("failed to parse bip39 phrase");
    let recovery_key = Key::from(mnemonic);

    let wrapped_master_hex = "caec81365f37d055c53545e74c34a35c8031d6e6810bfeebbc119c7c34892176bbf19ae24c322ef6f9b385b1c5b01640c0cffaa2aa4827df9e054e12e73251c2f7d7713c778a2d14bacaf587b01ef2bd51d3a883d4cba0c6dc0e6e702db7568f";

    let wrapped_master = WrappedKey::from_hex(wrapped_master_hex.to_owned(), KeyType::Master).expect("Failed to decode key hex");
    assert!(wrapped_master.bytes.len() == 48, "invalid wrapped master key length");

    let master = match wrapped_master.to_key(&recovery_key, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };

    assert!(master.bytes.len() == 32, "invalid master key length");
}

#[test]
fn key_unwrap_lowercase_keyset_generated_by_sdk_00600_test() {
    let phrase = "special cheap live sing proud ethics public you apology outside empty person";

    let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()).expect("failed to parse bip39 phrase");
    let recovery_key = Key::from(mnemonic);

    let wrapped_master_hex = "caec81365f37d055c53545e74c34a35c8031d6e6810bfeebbc119c7c34892176bbf19ae24c322ef6f9b385b1c5b01640c0cffaa2aa4827df9e054e12e73251c2f7d7713c778a2d14bacaf587b01ef2bd51d3a883d4cba0c6dc0e6e702db7568f";
    let wrapped_main_hex = "405975cfb94e00fb2fa794cc50d5d8389eaddb224edd4f92fd739a6821af8e999f69736f1c8a69e40c3dede2f1eefaac6144fd143065a43469304b56ecb3160d92e4c280895607752347a70d541ef0259f7347644f913c1fc8c87dcc9ca68104";
    let wrapped_hmac_hex = "4dbc87f0ba2d5e6ad37c2fa86d790df01957ab1f4ea5055704ce8f27602c985686316c9c0811b4fa36d871e67221322918f1e242ada5b268c32124d8873d8683ec67d5512f5f1b38aa614e98768565f7d98333146a231c8a803a9aeaa220ad3b";
    let wrapped_tweak_hex = "abf16c9cd3516db370731b6377b4b54accc804502e1ca53666d411f4e8264b7989a75b3c97584f9b6e18c449f03bd999e92cf5aaeb03024111e1989072ee9830b4a76e8440e493861acff7a6efcfae648e1d4bcc7fc0f28509710caeb87cce1e";



    let wrapped_master = WrappedKey::from_hex(wrapped_master_hex.to_owned(), KeyType::Master).expect("Failed to decode key hex");
    assert!(wrapped_master.bytes.len() == 48, "invalid wrapped master key length");

    let wrapped_main = WrappedKey::from_hex(wrapped_main_hex.to_owned(), KeyType::Main).expect("Failed to decode key hex");
    assert!(wrapped_main.bytes.len() == 48, "invalid wrapped main key length");

    let wrapped_hmac = WrappedKey::from_hex(wrapped_hmac_hex.to_owned(), KeyType::HMAC).expect("Failed to decode key hex");
    assert!(wrapped_hmac.bytes.len() == 48, "invalid wrapped hmac key length");

    let wrapped_tweak = WrappedKey::from_hex(wrapped_tweak_hex.to_owned(), KeyType::Tweak).expect("Failed to decode key hex");
    assert!(wrapped_tweak.bytes.len() == 48, "invalid wrapped tweak key length");

    // check the master key

    let master = match wrapped_master.to_key(&recovery_key, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };

    assert!(master.bytes.len() == 32, "invalid master key length");

    // check the main key

    let main = match wrapped_main.to_key(&master, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return
        },
    };

    assert!(main.bytes.len() == 32, "invalid main key length");

    // check the hmac key

    let hmac = match wrapped_hmac.to_key(&master, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return
        },
    };

    assert!(hmac.bytes.len() == 32, "invalid hmac key length");

    // check the tweak key

    let tweak = match wrapped_tweak.to_key(&master, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return
        },
    };

    assert!(tweak.bytes.len() == 32, "invalid tweak key length");
}

#[test]
fn key_unwrap_corrupt_hex_generated_by_sdk_00600_test() {
    let phrase = "special cheap live sing proud ethics public you apology outside empty person";

    let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()).expect("failed to parse bip39 phrase");
    let recovery_key = Key::from(mnemonic);

    let original_master_wrapped_hex = "caec81365f37d055c53545e74c34a35c8031d6e6810bfeebbc119c7c34892176bbf19ae24c322ef6f9b385b1c5b01640c0cffaa2aa4827df9e054e12e73251c2f7d7713c778a2d14bacaf587b01ef2bd51d3a883d4cba0c6dc0e6e702db7568f";
    let corrupt_master_wrapped_hex =  "000000000000000000000000000000000000000000000000bc119c7c34892176bbf19ae24c322ef6f9b385b1c5b01640c0cffaa2aa4827df9e054e12e73251c2f7d7713c778a2d14bacaf587b01ef2bd51d3a883d4cba0c6dc0e6e702db7568f";

    let original_master_wrapped = WrappedKey::from_hex(original_master_wrapped_hex.to_owned(), KeyType::Master).expect("Failed to decode key");
    assert!(original_master_wrapped.bytes.len() == 48, "invalid original wrapped master key length");

    let original_master = match original_master_wrapped.to_key(&recovery_key, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };
    assert!(original_master.bytes.len() == 32, "invalid original master key length");


    let corrected_master_wrapped = WrappedKey::from_hex(corrupt_master_wrapped_hex.to_owned(), KeyType::Master).expect("Failed to decode key");
    assert!(corrected_master_wrapped.bytes.len() == 48, "invalid corrected wrapped master key length");

    let corrected_master = match corrected_master_wrapped.to_key(&recovery_key, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };

    assert!(corrected_master.bytes.len() == 32, "invalid corrected master key length");

    assert!(corrected_master.bytes == original_master.bytes, "corrected master key bytes do not match original");
}

#[test]
fn key_unwrap_corrupt_ecc_generated_by_sdk_00600_test() {
    let phrase = "special cheap live sing proud ethics public you apology outside empty person";

    let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()).expect("failed to parse bip39 phrase");
    let recovery_key = Key::from(mnemonic);

    let original_master_wrapped_hex = "caec81365f37d055c53545e74c34a35c8031d6e6810bfeebbc119c7c34892176bbf19ae24c322ef6f9b385b1c5b01640c0cffaa2aa4827df9e054e12e73251c2f7d7713c778a2d14bacaf587b01ef2bd51d3a883d4cba0c6dc0e6e702db7568f";
    let corrupt_master_wrapped_hex =  "caec81365f37d055c53545e74c34a35c8031d6e6810bfeebbc119c7c34892176bbf19ae24c322ef6f9b385b1c5b01640c0cffaa2aa4827df9e054e12e73251c2f7d7713c778a2d14000000000000000000000000000000000000000000000000";

    let original_master_wrapped = WrappedKey::from_hex(original_master_wrapped_hex.to_owned(), KeyType::Master).expect("Failed to decode key");
    assert!(original_master_wrapped.bytes.len() == 48, "invalid original wrapped master key length");

    let original_master = match original_master_wrapped.to_key(&recovery_key, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };
    assert!(original_master.bytes.len() == 32, "invalid original master key length");


    let corrected_master_wrapped = WrappedKey::from_hex(corrupt_master_wrapped_hex.to_owned(), KeyType::Master).expect("Failed to decode key");
    assert!(corrected_master_wrapped.bytes.len() == 48, "invalid corrected wrapped master key length");

    let corrected_master = match corrected_master_wrapped.to_key(&recovery_key, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };

    assert!(corrected_master.bytes.len() == 32, "invalid corrected master key length");

    assert!(corrected_master.bytes == original_master.bytes, "corrected master key bytes do not match original");
}

#[test]
#[should_panic]
fn key_unwrap_corrupt_ecc_too_damaged_generated_by_sdk_00600_test() {
    let phrase = "special cheap live sing proud ethics public you apology outside empty person";

    let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()).expect("failed to parse bip39 phrase");
    let recovery_key = Key::from(mnemonic);

    let original_master_wrapped_hex = "caec81365f37d055c53545e74c34a35c8031d6e6810bfeebbc119c7c34892176bbf19ae24c322ef6f9b385b1c5b01640c0cffaa2aa4827df9e054e12e73251c2f7d7713c778a2d14bacaf587b01ef2bd51d3a883d4cba0c6dc0e6e702db7568f";
    let corrupt_master_wrapped_hex =  "caec81365f37d055c53545e74c34a35c8031d6e6810bfeebbc119c7c34892176bbf19ae24c322ef6f9b385b1c5b01640c0cffaa2aa4827df9e054e12e73251c2f7d7713c00000000000000000000000000000000000000000000000000000000";

    let original_master_wrapped = WrappedKey::from_hex(original_master_wrapped_hex.to_owned(), KeyType::Master).expect("Failed to decode key");
    assert!(original_master_wrapped.bytes.len() == 48, "invalid original wrapped master key length");

    let original_master = match original_master_wrapped.to_key(&recovery_key, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };
    assert!(original_master.bytes.len() == 32, "invalid original master key length");


    let corrected_master_wrapped = WrappedKey::from_hex(corrupt_master_wrapped_hex.to_owned(), KeyType::Master).expect("Failed to decode key");
    assert!(corrected_master_wrapped.bytes.len() == 48, "invalid corrected wrapped master key length");

    let corrected_master = match corrected_master_wrapped.to_key(&recovery_key, None) {
        Ok(k) => k,
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };

    assert!(corrected_master.bytes.len() == 32, "invalid corrected master key length");

    assert!(corrected_master.bytes == original_master.bytes, "corrected master key bytes do not match original");
}

#[test]
#[should_panic]
fn key_unwrap_failing_master_test() {
    let phrase = "hockey increase universe rib stone nothing someone sentence click game fresh vessel";

    let mnemonic = Bip39::from_mnemonic(phrase.to_string(), Language::English, "".to_string()).expect("failed to parse bip39 phrase");
    let recovery_key = Key::from(mnemonic);

    let wrapped_master_hex = "9e1215400656d9c56cfe62b03d0b949fed1200bad51fc23477fb10623a2f6b767deb3553e68de9dd0141bf14e58fa890";

    let wrapped_master = WrappedKey::from_hex(wrapped_master_hex.to_owned(), KeyType::Master).expect("Failed to decode key hex");
    match wrapped_master.to_key(&recovery_key, None) {
        Ok(_) => {},
        Err(e) => {
            assert!(true == false, format!("{}", e));
            return;
        },
    };
}
