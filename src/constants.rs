pub static BLOCK_DIRECTORIES: &'static[&'static str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"];

pub static HMAC_SIZE: i32  = 32; // 32 bytes
pub static NONCE_SIZE: i32 = 24; // 24 bytes
pub static WRAPPED_KEY_SIZE: i32 = 48; // 32 bytes + 16 bytes auth tag = 48 bytes

pub static DEFAULT_PERMISSIONS: i32 = 493;

pub static DEBUG_STATISTICS: bool = false;