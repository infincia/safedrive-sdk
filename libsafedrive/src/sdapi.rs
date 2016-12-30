use std;
use std::collections::hash_map::HashMap;
use std::io::Read;

extern crate reqwest;
extern crate serde;
extern crate serde_json;

header! { (SDAuthToken, "SD-Auth-Token") => [String] }

use util::*;
use error::*;
use models::*;

// SD API

pub fn client_register<S, T>(email: S, password: T) -> Result<(Token, UniqueClientID), SDAPIError> where S: Into<String>, T: Into<String> {

    let em = email.into();
    let pa = password.into();

    let uid = match unique_client_hash(&em) {
        Ok(hash) => hash,
        Err(e) => e,
    };


    let mut map = HashMap::new();

    if cfg!(target_os="windows") {
        map.insert("operatingSystem", "Windows");
    } else {
        map.insert("operatingSystem", "OS X");
    }
    map.insert("email", &em);
    map.insert("password", &pa);
    map.insert("language", "en-US");


    map.insert("uniqueClientId", &uid);

    let client = reqwest::Client::new().unwrap();
    let mut result = try!(client.post("https://safedrive.io/api/1/client/register").json(&map).send());
    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    let token: Token = match serde_json::from_str(&response) {
        Ok(token) => token,
        Err(_) => return Err(SDAPIError::RequestFailed)
    };

    let u = UniqueClientID { id: uid.to_owned() };

    Ok((token, u))
}

pub fn account_status(token: &Token) -> Result<AccountStatus, SDAPIError> {
    let client = reqwest::Client::new().unwrap();
    let request = client.get("https://safedrive.io/api/1/account/status")
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());
    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    let account_status: AccountStatus = match serde_json::from_str(&response) {
        Ok(a) => a,
        Err(_) => return Err(SDAPIError::RequestFailed)
    };

    Ok(account_status)
}

pub fn account_details(token: &Token) -> Result<AccountDetails, SDAPIError> {
    let client = reqwest::Client::new().unwrap();
    let request = client.get("https://safedrive.io/api/1/account/details")
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());
    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    let account_details: AccountDetails = match serde_json::from_str(&response) {
        Ok(a) => a,
        Err(_) => return Err(SDAPIError::RequestFailed)
    };

    Ok(account_details)
}

pub fn account_key(master: String, main: String, hmac: String) -> Result<(String, String, String), SDAPIError> {


    let mut map = HashMap::new();
    map.insert("master", master);
    map.insert("main", main);
    map.insert("hmac", hmac);

    let client = reqwest::Client::new().unwrap();
    let request = client.post("https://safedrive.io/api/1/account/key")
        .json(&map);
        //.header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());
    let mut response = String::new();

    try!(result.read_to_string(&mut response));

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