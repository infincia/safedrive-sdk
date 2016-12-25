use std;
use std::collections::hash_map::HashMap;
use std::io::Read;

extern crate reqwest;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;

use self::rustc_serialize::hex::{ToHex, FromHex, FromHexError};


use util::*;

#[derive(Debug)]
pub enum SDAPIError {
    RequestFailed
}

impl From<std::io::Error> for SDAPIError {
    fn from(err: std::io::Error) -> SDAPIError {
        match err {
            _ => SDAPIError::RequestFailed
        }
    }
}

impl From<self::reqwest::Error> for SDAPIError {
    fn from(err: self::reqwest::Error) -> SDAPIError {
        match err {
            _ => SDAPIError::RequestFailed
        }
    }
}

impl From<self::serde_json::Error> for SDAPIError {
    fn from(err: self::serde_json::Error) -> SDAPIError {
        match err {
            _ => SDAPIError::RequestFailed
        }
    }
}







// SD API

pub fn client_register(email: &String, password: &String) -> Result<String, String> {

    let uid = match unique_client_hash(&email) {
        Ok(hash) => hash,
        Err(e) => e,
    };


    let mut map = HashMap::new();

    if cfg!(target_os="windows") {
        map.insert("operatingSystem", "Windows");
    } else {
        map.insert("operatingSystem", "OS X");
    }
    map.insert("email", email);
    map.insert("password", password);
    map.insert("language", "en-US");


    map.insert("uniqueClientId", &uid);

    let client = reqwest::Client::new().unwrap();
    let result = client.post("https://safedrive.io/api/1/client/register").json(&map).send();
    let mut response = String::new();

    let _ = result.expect("didn't get response object").read_to_string(&mut response).expect("couldn't read response");
    let response_map: self::serde_json::Map<String, String> = serde_json::from_str(&response).unwrap();
    match response_map.get("token") {
        Some(token) => Ok(token.clone()),
        None => Err(format!("failed to get token"))
    }

}

pub fn account_key(master: String, main: String, hmac: String) -> Result<(String, String, String), SDAPIError> {


    let mut map = HashMap::new();
    map.insert("master", master);
    map.insert("main", main);
    map.insert("hmac", hmac);

    let client = reqwest::Client::new().unwrap();
    let result = client.post("https://safedrive.io/api/1/account/key").json(&map).send();
    let mut response = String::new();

    let mut r = try!(result);
    try!(r.read_to_string(&mut response));

    let response_map: self::serde_json::Map<String, String> = try!(serde_json::from_str(&response));
    let real_master = match response_map.get("master") {
        Some(k) => k.clone(),
        None => return Err(SDAPIError::RequestFailed)
    };
    let real_main = match response_map.get("main") {
        Some(k) => k.clone(),
        None => return Err(SDAPIError::RequestFailed)
    };
    let real_hmac = match response_map.get("hmac") {
        Some(k) => k.clone(),
        None => return Err(SDAPIError::RequestFailed)
    };


    Ok((real_master, real_main, real_hmac))
}