use std::io::{Read, Write};

use ::error::{CryptoError, SDError};
use ::binformat::BinaryFormat;
use ::nom::IResult::*;
use ::keys::{Key, WrappedKey, KeyType};
use ::models::*;

use ::constants::*;
use ::CHANNEL;

#[derive(Deserialize, Debug, Clone)]
pub struct SyncSession {
    #[serde(skip_deserializing)]
    version: SyncVersion,
    pub folder_id: Option<u64>,
    pub name: String,
    pub size: Option<u64>,
    pub time: Option<u64>,
    pub id: Option<u64>,
    #[serde(skip_deserializing)]
    channel: Channel,
    #[serde(skip_deserializing)]
    production: bool,
    #[serde(skip_deserializing)]
    data: Vec<u8>,
    #[serde(skip_deserializing)]
    compressed: bool,
    #[serde(skip_deserializing)]
    real_size: u64,
    #[serde(skip_deserializing)]
    compressed_size: Option<u64>,
}

impl SyncSession {
    pub fn new(version: SyncVersion, folder_id: u64, name: String, size: Option<u64>, time: Option<u64>, data: Vec<u8>) -> SyncSession {
        let real_size = data.len() as u64;

        match version {
            SyncVersion::Version1 | SyncVersion::Version2 => {},
            _ => {
                panic!("Attempted to create invalid session version");
            },
        };

        let (compressed, maybe_compressed_data, maybe_compressed_size) = match version {
            SyncVersion::Version1 => {
                /// no compression

                (false, data, None)
            },
            SyncVersion::Version2 => {
                let buf = Vec::new();
                let mut encoder = ::lz4::EncoderBuilder::new().level(16).build(buf).unwrap();
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

        SyncSession { version: version, folder_id: Some(folder_id), name: name, size: size, time: time, data: maybe_compressed_data, compressed: compressed, id: None, production: production, channel: channel, real_size: real_size, compressed_size: maybe_compressed_size }
    }


    pub fn to_wrapped(self, main: &Key) -> Result<WrappedSyncSession, CryptoError> {

        /// generate a new session key
        let session_key = Key::new(KeyType::Session);

        /// We use a random nonce here because we don't need to know what it is in advance, unlike blocks
        ///
        /// We use the same nonce both while wrapping the session key, and the session data itself
        /// this is safe because using the same nonce with 2 different keys is not nonce reuse
        let session_nonce = ::sodiumoxide::crypto::secretbox::gen_nonce();

        /// get the session data, padded and prefixed with a u32 length as little endian
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
                panic!("Attempted to wrap invalid session version");
            },
        };

        /// encrypt the data using the session key
        let wrapped_data = ::sodiumoxide::crypto::secretbox::seal(&to_encrypt, &session_nonce, &session_key.as_sodium_secretbox_key());

        /// wrap the session key with the main encryption key
        let wrapped_session_key = match session_key.to_wrapped(main, Some(&session_nonce)) {
            Ok(wk) => wk,
            Err(e) => return Err(e),
        };

        Ok(WrappedSyncSession { version: self.version, folder_id: self.folder_id, name: self.name, size: self.size, time: self.time, wrapped_data: wrapped_data, wrapped_key: wrapped_session_key, nonce: session_nonce, compressed: self.compressed, production: self.production, channel: self.channel })
    }

    pub fn compressed(&self) -> bool {
        self.compressed
    }

    pub fn real_size(&self) -> u64 {
        self.real_size
    }

    pub fn compressed_size(&self) -> Option<u64> {
        self.compressed_size
    }
}

impl AsRef<[u8]> for SyncSession {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<'a> ::std::fmt::Display for SyncSession {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "SyncSession <version:{}, channel:{}, production:{}, compressed:{}>", self.version, self.channel, self.production, self.compressed)
    }
}


#[derive(Debug, Clone)]
pub struct WrappedSyncSession {
    version: SyncVersion,
    pub folder_id: Option<u64>,
    pub name: String,
    pub size: Option<u64>,
    pub time: Option<u64>,
    wrapped_data: Vec<u8>,
    channel: Channel,
    production: bool,
    nonce: ::sodiumoxide::crypto::secretbox::Nonce,
    wrapped_key: WrappedKey,
    compressed: bool,
}


impl WrappedSyncSession {
    pub fn to_session(self, main: &Key) -> Result<SyncSession, SDError> {

        let session_key = match self.wrapped_key.to_key(main, Some(&self.nonce)) {
            Ok(k) => k,
            Err(_) => return Err(SDError::CryptoError(CryptoError::SessionDecryptFailed))
        };

        let session_raw = match ::sodiumoxide::crypto::secretbox::open(&self.wrapped_data, &self.nonce, &session_key.as_sodium_secretbox_key()) {
            Ok(s) => s,
            Err(_) => return Err(SDError::CryptoError(CryptoError::SessionDecryptFailed))
        };

        let unpadded_data = match self.version {
            SyncVersion::Version1 => {
                session_raw
            },
            SyncVersion::Version2 => {
                let unpadded = match ::binformat::remove_padding(&session_raw) {
                    Done(_, o) => o,
                    Error(e) => {
                        debug!("session padding removal failed: {}", &e);

                        match e {
                            ::nom::verbose_errors::Err::Code(ref kind) => {
                                debug!("session padding removal failure kind: {:?}", kind);

                            },
                            ::nom::verbose_errors::Err::Node(ref kind, ref err) => {
                                debug!("session padding removal failure node: {:?}, {}", kind, err);

                            },
                            ::nom::verbose_errors::Err::Position(ref kind, ref position) => {
                                debug!("session padding removal failure position: {:?}: {:?}", kind, position);

                            },
                            ::nom::verbose_errors::Err::NodePosition(ref kind, ref position, ref err) => {
                                debug!("session padding removal failure kind: {:?}: {:?}, {}", kind, position, err);

                            },
                        };

                        return Err(SDError::SessionUnreadable)
                    },
                    Incomplete(_) => {
                        debug!("session data cannot be unpadded, this should never happen");
                        return Err(SDError::SessionUnreadable)
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
                        let mut uncompressed_data = Vec::new();
                        let mut decoder = ::lz4::Decoder::new(unpadded_data.as_slice()).unwrap();
                        let _ = decoder.read_to_end(&mut uncompressed_data);

                        let compressed_size = unpadded_data.len();


                        (uncompressed_data, Some(compressed_size as u64))
                    },
                    false => {
                        (unpadded_data, None)
                    }
                }
            },
            _ => panic!("unknown binary version")
        };

        let real_size = maybe_uncompressed_data.len();

        Ok(SyncSession { version: self.version, folder_id: self.folder_id, name: self.name, size: self.size, real_size: real_size as u64, compressed_size: maybe_compressed_size, time: self.time, data: maybe_uncompressed_data, compressed: self.compressed, id: None, production: self.production, channel: self.channel })
    }

    pub fn to_binary(self) -> Vec<u8> {

        let mut binary_data = Vec::new();

        /// first 8 bytes are the file ID, type, version, flags, and 2 byte reserved area
        let magic: &'static [u8; 2] = br"sd";
        let file_type: &'static [u8; 1] = br"s";
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

        /// next 48 bytes will be the wrapped session key
        binary_data.extend(self.wrapped_key.as_ref());

        /// next 24 bytes will be the nonce
        binary_data.extend(self.nonce.as_ref());
        assert!(binary_data.len() == magic.len() + file_type.len() + version.len() + flag_ref.len() + reserved.len() + (SECRETBOX_KEY_SIZE + SECRETBOX_MAC_SIZE) + SECRETBOX_NONCE_SIZE);

        /// remainder will be the encrypted session data
        binary_data.extend(self.wrapped_data);

        binary_data
    }

    pub fn from(body: SyncSessionResponse) -> Result<WrappedSyncSession, SDError> {

        let raw_session: BinaryFormat = match ::binformat::binary_parse(&body.chunk_data) {
            Done(_, o) => o,
            Error(e) => {
                debug!("session parsing failed: {}", &e);

                match e {
                    ::nom::verbose_errors::Err::Code(ref kind) => {
                        debug!("session parsing failure kind: {:?}", kind);

                    },
                    ::nom::verbose_errors::Err::Node(ref kind, ref err) => {
                        debug!("session parsing failure node: {:?}, {}", kind, err);

                    },
                    ::nom::verbose_errors::Err::Position(ref kind, ref position) => {
                        debug!("session parsing failure position: {:?}: {:?}", kind, position);

                    },
                    ::nom::verbose_errors::Err::NodePosition(ref kind, ref position, ref err) => {
                        debug!("session parsing failure kind: {:?}: {:?}, {}", kind, position, err);

                    },
                };

                return Err(SDError::SessionUnreadable)
            },
            Incomplete(_) => {
                debug!("session file cannot be parsed, this should never happen");
                return Err(SDError::SessionUnreadable)
            }
        };

        debug!("got valid binary file: {}", &raw_session);

        let session_ver = match raw_session.version {
            "01" => SyncVersion::Version1,
            "02" => SyncVersion::Version2,
            _ => panic!("Invalid binary session version")
        };
        let wrapped_session_key_raw = raw_session.wrapped_key.to_vec();
        let nonce_raw = raw_session.nonce;

        let session_nonce = ::sodiumoxide::crypto::secretbox::Nonce::from_slice(&nonce_raw)
            .expect("failed to get nonce");

        let wrapped_session_raw = raw_session.wrapped_data.to_vec();

        let wrapped_session_key = WrappedKey::from(wrapped_session_key_raw, KeyType::Session);

        let channel = raw_session.channel;

        let production = raw_session.production;
        let compressed = raw_session.compressed;

        let wrapped_session = WrappedSyncSession { version: session_ver, folder_id: Some(body.folder_id), name: body.name.to_string(), size: None, time: None, wrapped_key: wrapped_session_key, wrapped_data: wrapped_session_raw, nonce: session_nonce, compressed: compressed, channel: channel, production: production };


        debug!("got valid wrapped session: {}", &wrapped_session);

        Ok(wrapped_session)
    }

    pub fn compressed(&self) -> bool {
        self.compressed
    }

}

impl AsRef<[u8]> for WrappedSyncSession {
    fn as_ref(&self) -> &[u8] {
        self.wrapped_data.as_ref()
    }
}

impl<'a> ::std::fmt::Display for WrappedSyncSession {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "WrappedSyncSession <version:{}, channel:{}, production:{}, compressed:{}>", self.version, self.channel, self.production, self.compressed)
    }
}