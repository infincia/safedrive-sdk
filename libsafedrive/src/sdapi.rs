use std::io::Read;

extern crate reqwest;
extern crate serde;
extern crate serde_json;

header! { (SDAuthToken, "SD-Auth-Token") => [String] }
header! { (ContentType, "Content-Type: multipart/form-data; boundary=") => [String] }
header! { (ContentLength, "Content-Length: ") => [usize] }

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
    let map_req = ClientRegisterRequest { operatingSystem: os.to_string(), email: em, password: pa, language: "en-US".to_string(), uniqueClientId: uid.clone() };

    let client = reqwest::Client::new().unwrap();
    let request = client.post("https://safedrive.io/api/1/client/register")
        .json(&map_req);

    let mut result = try!(request.send());
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

pub fn account_key(token: &Token, master: String, main: String, hmac: String) -> Result<(String, String, String), SDAPIError> {

    let map_req = WrappedKeyset { master: master, main: main, hmac: hmac };

    let client = reqwest::Client::new().unwrap();
    let request = client.post("https://safedrive.io/api/1/account/key")
        .json(&map_req)
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    let wrapped_keyset: WrappedKeyset = try!(serde_json::from_str(&response));

    Ok((wrapped_keyset.master, wrapped_keyset.main, wrapped_keyset.hmac))
}

pub fn read_folders(token: &Token) -> Result<Vec<RegisteredFolder>, SDAPIError> {
    let client = reqwest::Client::new().unwrap();
    let request = client.get("https://safedrive.io/api/1/folder")
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());
    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    let folders: Vec<RegisteredFolder> = match serde_json::from_str(&response) {
        Ok(f) => f,
        Err(_) => return Err(SDAPIError::RequestFailed)
    };

    Ok(folders)
}

pub fn create_folder<S, T>(token: &Token, path: S, name: T, encrypted: bool) -> Result<u64, SDAPIError> where S: Into<String>, T: Into<String> {

    let pa = path.into();
    let na = name.into();

    let map_req = CreateFolderRequest { folderName: na, folderPath: pa, encrypted: encrypted };

    let client = reqwest::Client::new().unwrap();
    let request = client.post("https://safedrive.io/api/1/folder")
        .json(&map_req)
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());
    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    let folder_response: CreateFolderResponse = match serde_json::from_str(&response) {
        Ok(r) => r,
        Err(_) => return Err(SDAPIError::RequestFailed)
    };

    Ok(folder_response.id)
}