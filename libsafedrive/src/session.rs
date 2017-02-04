
use ::error::{CryptoError, SDError};
use ::binformat::BinaryFormat;
use ::nom::IResult::*;
use ::keys::{Key, WrappedKey, KeyType, KeyConversion};
use ::models::{SyncSessionResponse, SyncVersion};

use ::constants::*;


#[derive(Deserialize, Debug, Clone)]
pub struct SyncSession {
    #[serde(skip_deserializing)]
    version: SyncVersion,
    pub folder_id: Option<u64>,
    pub name: String,
    pub size: Option<u64>,
    pub time: Option<u64>,
    #[serde(skip_deserializing)]
    pub data: Vec<u8>,
}

impl SyncSession {
    pub fn new(version: SyncVersion, folder_id: u64, name: String, size: Option<u64>, time: Option<u64>, data: Vec<u8>) -> SyncSession {
        match version {
            SyncVersion::Version1 | SyncVersion::Version2 => {},
            _ => {
                panic!("Attempted to create invalid session version");
            },
        };


        SyncSession { version: version, folder_id: Some(folder_id), name: name, size: size, time: time, data: data }
    }


    pub fn to_wrapped(self, main: &Key) -> Result<WrappedSyncSession, CryptoError> {

        // generate a new session key
        let session_key_raw = ::sodiumoxide::randombytes::randombytes(SECRETBOX_KEY_SIZE);
        let session_key_struct = ::sodiumoxide::crypto::secretbox::Key::from_slice(&session_key_raw)
            .expect("failed to get session key struct");

        // We use a random nonce here because we don't need to know what it is in advance, unlike blocks
        let nonce_raw = ::sodiumoxide::randombytes::randombytes(SECRETBOX_NONCE_SIZE);

        let nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_raw)
            .expect("failed to get nonce");

        // we use the same nonce both while wrapping the session key, and the session data itself
        // this is safe because using the same nonce with 2 different keys is not nonce reuse

        // encrypt the session data using the session key
        let wrapped_data = ::sodiumoxide::crypto::secretbox::seal(&self.data, &nonce, &session_key_struct);

        // wrap the session key with the main encryption key
        let wrapped_session_key_raw = ::sodiumoxide::crypto::secretbox::seal(&session_key_raw, &nonce, &(main.as_sodium_secretbox_key()));
        assert!(wrapped_session_key_raw.len() == SECRETBOX_KEY_SIZE + SECRETBOX_MAC_SIZE);
        let wrapped_session_key = WrappedKey::new(wrapped_session_key_raw, KeyType::Session);


        Ok(WrappedSyncSession { version: self.version, folder_id: self.folder_id, name: self.name, size: self.size, time: self.time, wrapped_data: wrapped_data, wrapped_key: wrapped_session_key, nonce: nonce_raw })
    }
}




pub struct WrappedSyncSession {
    pub version: SyncVersion,
    pub folder_id: Option<u64>,
    pub name: String,
    pub size: Option<u64>,
    pub time: Option<u64>,
    pub wrapped_data: Vec<u8>,
    nonce: Vec<u8>,
    wrapped_key: WrappedKey,
}


impl WrappedSyncSession {
    pub fn to_session(self, main: &Key) -> Result<SyncSession, SDError> {
        let nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&self.nonce)
            .expect("failed to get nonce");

        let main_key_s = ::sodiumoxide::crypto::secretbox::Key::from_slice(main.as_ref())
            .expect("failed to get main key struct");

        let session_key_raw = match ::sodiumoxide::crypto::secretbox::open(self.wrapped_key.as_ref(), &nonce, &main_key_s) {
            Ok(k) => k,
            Err(_) => return Err(SDError::CryptoError(CryptoError::SessionDecryptFailed))
        };

        let session_key_s = ::sodiumoxide::crypto::secretbox::Key::from_slice(&session_key_raw).expect("failed to get unwrapped session key struct");

        let session_raw = match ::sodiumoxide::crypto::secretbox::open(&self.wrapped_data, &nonce, &session_key_s) {
            Ok(s) => s,
            Err(_) => return Err(SDError::CryptoError(CryptoError::SessionDecryptFailed))
        };


        Ok(SyncSession { version: self.version, folder_id: self.folder_id, name: self.name, size: self.size, time: self.time, data: session_raw })
    }

    pub fn to_binary(self) -> Vec<u8> {

        let mut binary_data = Vec::new();

        // first 8 bytes are the file ID, version, and reserved area
        let magic: &'static [u8; 2] = br"sd";
        let file_type: &'static [u8; 1] = br"s";
        let version = self.version.as_ref();
        let reserved: &'static [u8; 3] = br"000";

        binary_data.extend(magic.as_ref());
        binary_data.extend(file_type.as_ref());
        binary_data.extend(version);
        binary_data.extend(reserved.as_ref());

        // next 48 bytes will be the wrapped session key
        binary_data.extend(self.wrapped_key.as_ref());

        // next 24 bytes will be the nonce
        binary_data.extend(self.nonce);
        assert!(binary_data.len() == magic.len() + file_type.len() + version.len() + reserved.len() + (SECRETBOX_KEY_SIZE + SECRETBOX_MAC_SIZE) + SECRETBOX_NONCE_SIZE);

        // remainder will be the encrypted session data
        binary_data.extend(self.wrapped_data);

        binary_data
    }

    pub fn from(body: SyncSessionResponse) -> Result<WrappedSyncSession, SDError> {

        let raw_session: BinaryFormat = match ::binformat::binary_parse(&body.chunk_data) {
            Done(_, o) => o,
            Error(_) => return Err(SDError::SessionMissing),
            Incomplete(_) => panic!("should never happen")
        };

        debug!("got valid binary file: {}", &raw_session);

        let session_ver = match raw_session.version {
            "01" => SyncVersion::Version1,
            "02" => SyncVersion::Version2,
            _ => panic!("Invalid binary session version")
        };
        let wrapped_session_key_raw = raw_session.wrapped_key.to_vec();
        let nonce_raw = raw_session.nonce.to_vec();
        let wrapped_session_raw = raw_session.wrapped_data.to_vec();

        let wrapped_session_key = WrappedKey::new(wrapped_session_key_raw, KeyType::Session);

        Ok(WrappedSyncSession { version: session_ver, folder_id: Some(body.folder_id), name: body.name.to_string(), size: None, time: None, wrapped_key: wrapped_session_key, wrapped_data: wrapped_session_raw, nonce: nonce_raw })
    }

}