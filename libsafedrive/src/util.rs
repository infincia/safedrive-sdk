use std::path::{PathBuf, Path};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

// external crate imports

use ::rustc_serialize::hex::{ToHex};

use ::number_prefix::{binary_prefix, Standalone, Prefixed};

#[cfg(target_os = "macos")]
use ::objc_foundation::{NSObject, NSString, INSString};

#[cfg(target_os = "macos")]
use ::objc::runtime::{Class};

use ::byteorder::LittleEndian;
use ::byteorder::ByteOrder;

// internal imports

use ::constants::*;
use ::error::SDError;

pub fn generate_uuid() -> String {
    let sync_uuid = ::uuid::Uuid::new_v4().simple().to_string();

    sync_uuid
}

#[cfg(target_os = "macos")]
pub fn unique_client_hash(email: &str) -> Result<String, String> {
    let interface = "en0";

    if let Ok(hardware) = ::interfaces::Interface::get_by_name(interface) {
        if let Some(interface) = hardware {
            if let Ok(mac) = interface.hardware_addr() {
                let mac_string = mac.as_bare_string();
                let to_hash = mac_string + &email;
                let hash = sha256(to_hash.as_bytes());
                return Ok(hash)
            }
        }
    } else {
        error!("could not find {} interface", interface);
    }
    Err("failed to get mac address".to_string())
}

pub fn get_unique_client_id(local_storage_path: &Path) -> Result<String, SDError> {
    let mut uidp = PathBuf::from(local_storage_path);
    uidp.push(".unique_client_id");

    let mut f = try!(File::open(&uidp));

    let mut unique_client_id = String::new();

    try!(f.read_to_string(&mut unique_client_id));

    Ok(unique_client_id)
}

pub fn set_unique_client_id(unique_client_id: &str, local_storage_path: &Path) -> Result<(), SDError> {
    let mut uidp = PathBuf::from(local_storage_path);
    uidp.push(".unique_client_id");

    let mut f = try!(File::create(&uidp));

    try!(f.write_all(unique_client_id.as_bytes()));

    Ok(())
}

#[cfg(target_os = "macos")]
#[allow(non_snake_case)]
pub fn get_app_directory(config: &Configuration) -> Result<PathBuf, String> {
    let NSFileManager_cls = Class::get("NSFileManager").unwrap();
    let groupID = NSString::from_str(::constants::SD_GROUP_NAME);

    let mut storage_path = unsafe {
        let fm: *mut NSObject = msg_send![NSFileManager_cls, new];

        let groupContainerURL: *mut NSObject = msg_send![fm, containerURLForSecurityApplicationGroupIdentifier:&*groupID];

        let groupContainerPath: *mut NSString = msg_send![groupContainerURL, path];

        let s = (*groupContainerPath).as_str();

        let p = PathBuf::from(s);

        let _: () = msg_send![fm, release];
        /// these cause a segfault right now, but technically you would release these to avoid a leak
        //let _: () = msg_send![groupContainerURL, release];
        //let _: () = msg_send![groupContainerPath, release];

        p
    };
    match config {
        &Configuration::Staging => storage_path.push("staging"),
        &Configuration::Production => {},
    }
    match fs::create_dir_all(&storage_path) {
        Ok(()) => {},
        Err(_) => {}, // ignore this for the moment, it's primarily going to be the directory existing already
    }
    return Ok(storage_path)

}

#[cfg(not(target_os = "macos"))]
pub fn get_app_directory(config: &Configuration) -> Result<PathBuf, String> {


    let evar: &str;

    if cfg!(target_os="windows") {
        evar = "APPDATA";
    } else {
        /// probably linux, use home dir
        evar = "HOME";
    }
    let m = format!("failed to get {} environment variable", evar);
    let path = match ::std::env::var(evar) {
        Ok(e) => e,
        Err(_) => { return Err(m) }
    };

    let mut storage_path = PathBuf::from(&path);

    if cfg!(target_os="windows") {
        storage_path.push("SafeDrive");

    } else {
        storage_path.push(".safedrive");
    }

    match config {
        &Configuration::Staging => storage_path.push("staging"),
        &Configuration::Production => {},
    }
    match fs::create_dir_all(&storage_path) {
        Ok(()) => {},
        Err(_) => {}, // ignore this for the moment, it's primarily going to be the directory existing already
    }

    return Ok(storage_path)
}

pub fn get_openssl_directory() -> Result<PathBuf, ()> {
    let output = match Command::new("openssl")
        .arg("version")
        .arg("-d")
        .output() {
        Ok(o) => o,
        Err(_) => return Err(()),
    };

    let out = output.stdout;

    let s = match String::from_utf8(out) {
        Ok(s) => s,
        Err(_) => return Err(()),
    };

    let re = match ::regex::Regex::new(r#"OPENSSLDIR: "(.*)""#) {
        Ok(re) => re,
        Err(_) => return Err(()),
    };

    let cap = match re.captures(&s) {
        Some(cap) => cap,
        None => return Err(()),
    };

    let r = &cap[1];

    let path = PathBuf::from(r);

    return Ok(path)
}


pub fn get_current_os() -> &'static str {

    let os: &str;

    if cfg!(target_os="windows") {
        os = "Windows";
    } else if cfg!(target_os="macos") {
        os = "OS X";
    } else if cfg!(target_os="linux") {
        os = "Linux";
    } else {
        os = "Unknown";
    }

    os
}

// crypto helpers

pub fn sha256(input: &[u8]) -> String {
    let hashed = ::sodiumoxide::crypto::hash::sha256::hash(input);
    let h = hashed.as_ref().to_hex();

    h
}

/// Add a length prefix and zero-padding to a slice of bytes
///
/// Used to pad the unencrypted data chunks we generate from files before encryption
///
/// Without padding, it is easier to guess whether a user has stored a specific, known file by
/// looking for pre-calculated patterns in the sizes of blocks stored in a user's account.
///
/// Padding doesn't solve the problem entirely, only reduces the probability of a guess being
/// correct. Storing a large number of blocks should greatly reduce that probability.
///
/// It can't be prevented entirely without breaking any relationship between the data
/// and the sizes, as the sizes are a function of the hash value of the data. Attempts to
/// break that relationship using secret parameters with the hash function may cause it to be
/// obscured, but won't have any effect at all on files that are not large enough to contain a
/// boundary or trigger the minimum chunk size. Those files would be stored at their actual sizes
/// without some form of padding.
///
/// For an example of why padding doesn't solve the problem entirely, if a block is stored padded to
/// 1024 bytes, there is still a 1 in 128 (2^7) chance that it was any specific size between 896 and
/// 1023. We progressively increase that by 1-bit at specific sizes to trade overhead for safety
/// margin, but it's still fairly small.
///
/// For safety, we would prefer to pad all binary files to the same size, but that would require
/// limiting the maximum size of blocks to a relatively small size, or incurring extreme overhead
/// for smaller blocks.
///
///
/// Parameters:
///
///     input: a slice of unencrypted data to be prefixed with the length of the original data and
///            then padded with zeros
///
/// # Examples
///
/// ```
/// let v = vec![1u8, 2u8, 3u8, 4u8];
/// let padded_v = pad_and_prefix_length(v.as_slice());
///
/// ```
///
pub fn pad_and_prefix_length(input: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();

    /// add the length of the input data as little endian u32 in the first 4 bytes
    let input_length = input.len();

    assert!(input_length <= u32::max_value() as usize);


    let mut size_buf = [0; 4];
    LittleEndian::write_u32(&mut size_buf, input_length as u32);
    buf.extend(&size_buf);

    /// add the input data
    buf.extend(input);

    /// determine what to round up to for the input size
    ///
    /// we don't actually need to map all of these, they're just here to keep the worst case overhead
    /// visible
    let round = match input_length {
           0...128  => 128,  // potentially 99%, but necessary and shouldn't affect the average much
         128...256  => 128,  // 50%
         256...384  => 128,  // 33%
         384...512  => 128,  // 33%
         512...640  => 128,  // 25%
         640...768  => 128,  // 20%
         768...896  => 128,  // 16.5%
         896...1024 => 128,  // 14.2%
        1024...1152 => 128,  // 12.5%
        1152...1280 => 128,  // 11%
        1280...1408 => 128,  // 10%
        1408...1536 => 128,  // 9%,
        1536...1664 => 128,  // 8.3%,
        1664...1792 => 128,  // 7.69%,
        1792...1920 => 128,  // 7.14%,
        1920...2048 => 128,  // 6.6%,
        2048...2304 => 256,  // 12.5%
        2304...2560 => 256,  // 11%
        2560...2816 => 256,  // 10%
        2816...3072 => 256,  // 9%
        3072...3328 => 256,  // 8.3%
        3328...3584 => 256,  // 7.69%
        3584...3840 => 256,  // 7.14%
        3840...4096 => 256,  // 6.6%
        4096...4608 => 512,  // 12.5%
        4608...5120 => 512,  // 11%
        5120...5632 => 512,  // 10%
        5632...6114 => 512,  // 9%
        6114...6656 => 512,  // 8.3%
        _ => 512,            // less than 7.69%, average closer to 1-2%
    };

    /// check to see how much padding we need
    let nearest = nearest_to(input_length, round);
    let padding_needed = nearest - input_length;

    /// pad the end of the data with zeros if needed
    buf.resize(size_buf.len() + input_length + padding_needed, 0u8);

    buf
}


#[test]
pub fn pad_4bytes() {
    let v = vec![1u8, 2u8, 3u8, 4u8];
    assert!(v.len() == 4);

    let padded_v = pad_and_prefix_length(v.as_slice());

    assert!(padded_v.len() == 4 + 128);
}

/// format helpers

pub fn pretty_bytes(input: f64) -> String {
    match binary_prefix(input) {
        Standalone(bytes)   => format!("{} bytes", bytes),
        Prefixed(prefix, n) => format!("{:.2} {}B", n, prefix),
    }
}

/// math helpers

pub fn timestamp_to_ms(sec: i64, ms: u32) -> i64 {
    (sec * 1000) + (ms as i64)
}

/// Get the next highest integer after `num` that is a multiple of `multiple`
///
/// Parameters:
///
///     num: any integer that will fit in a u32
///
///     multiple: the next standardized size to look for
///
pub fn nearest_to(num: usize, multiple: usize) -> usize {
    (num + multiple - 1) & !(multiple - 1) as usize
}

#[test]
fn nearest_to_60() {
    assert!(nearest_to(60, 128) == 128);
}

#[test]
fn nearest_to_405() {
    assert!(nearest_to(405, 512) == 512);
}

#[test]
fn nearest_to_1890() {
    assert!(nearest_to(1890, 512) == 2048);
}


#[cfg(target_os = "linux")]
#[test]
fn openssl_directory_test() {
    match get_openssl_directory() {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };
}



#[test]
fn staging_directory_test() {
    match get_app_directory(&Configuration::Staging) {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };
}

#[test]
fn production_directory_test() {
    match get_app_directory(&Configuration::Production) {
        Ok(p) => p,
        Err(_) => { assert!(true == false); return }
    };
}