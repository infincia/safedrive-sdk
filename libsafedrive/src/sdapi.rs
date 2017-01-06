#![allow(non_snake_case)]

use std::io::Read;

extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate hyper;

header! { (SDAuthToken, "SD-Auth-Token") => [String] }
header! { (ContentType, "Content-Type: multipart/form-data; boundary=") => [String] }
header! { (ContentLength, "Content-Length: ") => [usize] }

use util::*;
use error::*;
use models::*;
use constants::*;

#[derive(Serialize, Debug)]
pub enum APIRequestBody<'a> {
    RegisterClient { operatingSystem: &'a str,
                               email: &'a str,
                            password: &'a str,
                            language: &'a str,
                      uniqueClientId: &'a str },
    AccountKey { master: &'a str,
                   main: &'a str,
                   hmac: &'a str },
    CreateFolder { folderName: &'a str,
                   folderPath: &'a str,
                    encrypted: bool },
    RegisterSyncSession { folder_id: i32,
                             name: &'a str,
                        encrypted: bool },
    FinishSyncSession { session_data: &'a [u8] },
    AddBlock { chunk_data: &'a Option<Vec<u8>> }
}

pub enum APIEndpoint<'a> {
    RegisterClient { email: &'a str, password: &'a str, operatingSystem: &'a str, language: &'a str, uniqueClientId: &'a str },
    AccountStatus { token: &'a Token },
    AccountDetails { token: &'a Token },
    AccountKey { token: &'a Token, master: &'a str, main: &'a str, hmac: &'a str },
    ReadFolders { token: &'a Token },
    CreateFolder { token: &'a Token, path: &'a str, name: &'a str, encrypted: bool },
    RegisterSyncSession { token: &'a Token, folder_id: i32, name: &'a str, encrypted: bool },
    FinishSyncSession { token: &'a Token, folder_id: i32, name: &'a str, encrypted: bool, size: usize, session_data: &'a [u8] },
    ReadSyncSession { token: &'a Token, name: &'a str, encrypted: bool },
    ReadSyncSessions { token: &'a Token, encrypted: bool },
    CheckBlock { token: &'a Token, name: &'a str },
    WriteBlock { token: &'a Token, session: &'a str, name: &'a str, chunk_data: &'a Option<Vec<u8>> },
    ReadBlock { token: &'a Token, name: &'a str },

}

impl<'a> APIEndpoint<'a> {

    pub fn url(&self) -> String {
        let mut url = String::new();
        url += &self.protocol();
        url += &self.domain();
        url += &self.path();

        url
    }

    pub fn domain(&self) -> String {
        SDAPIDOMAIN_PRODUCTION.to_string()
    }

    pub fn protocol(&self) -> String {
        "https://".to_string()
    }

    pub fn method(&self) -> self::hyper::method::Method {
        match *self {
            APIEndpoint::RegisterClient { .. } => {
                self::hyper::method::Method::Post
            },
            APIEndpoint::AccountStatus { .. } => {
                self::hyper::method::Method::Get
            },
            APIEndpoint::AccountDetails { .. } => {
                self::hyper::method::Method::Get
            },
            APIEndpoint::AccountKey { .. } => {
                self::hyper::method::Method::Post
            },
            APIEndpoint::ReadFolders { .. } => {
                self::hyper::method::Method::Get
            },
            APIEndpoint::CreateFolder { .. } => {
                self::hyper::method::Method::Post
            },
            APIEndpoint::RegisterSyncSession { .. } => {
                self::hyper::method::Method::Post
            },
            APIEndpoint::FinishSyncSession { .. } => {
                self::hyper::method::Method::Post
            },
            APIEndpoint::ReadSyncSession { .. } => {
                self::hyper::method::Method::Get
            },
            APIEndpoint::ReadSyncSessions { .. } => {
                self::hyper::method::Method::Get
            },
            APIEndpoint::CheckBlock { .. } => {
                self::hyper::method::Method::Head
            },
            APIEndpoint::WriteBlock { .. } => {
                self::hyper::method::Method::Post
            },
            APIEndpoint::ReadBlock { .. } => {
                self::hyper::method::Method::Get
            },
        }
    }

    pub fn path(&self) -> String {
        let path = match *self {
            APIEndpoint::RegisterClient { .. } => {
                format!("/api/1/client/register")
            },
            APIEndpoint::AccountStatus { .. } => {
                format!("/api/1/account/status")
            },
            APIEndpoint::AccountDetails { .. } => {
                format!("/api/1/account/details")
            },
            APIEndpoint::AccountKey { .. } => {
                format!("/api/1/account/key")
            },
            APIEndpoint::ReadFolders { .. } => {
                format!("/api/1/folder")
            },
            APIEndpoint::CreateFolder { .. } => {
                format!("/api/1/folder")
            },
            APIEndpoint::RegisterSyncSession { folder_id, name, .. } => {
                format!("/api/1/sync/session/register/{}/{}", folder_id, name)
            },
            APIEndpoint::FinishSyncSession { name, size, .. } => {
                format!("/api/1/sync/session/{}/{}", name, size)
            },
            APIEndpoint::ReadSyncSession { name, .. } => {
                format!("/api/1/sync/session/{}", name)
            },
            APIEndpoint::ReadSyncSessions { .. } => {
                format!("/api/1/sync/session")
            },
            APIEndpoint::CheckBlock { name, .. } => {
                format!("/api/1/sync/block/{}", name)
            },
            APIEndpoint::WriteBlock { session, name, .. } => {
                format!("/api/1/sync/block/{}/{}", name, session)
            },
            APIEndpoint::ReadBlock { name, .. } => {
                format!("/api/1/sync/block/{}", name)
            },

        };

        path
    }

    pub fn body(&self) -> Option<APIRequestBody> {
        match *self {
            APIEndpoint::RegisterClient { operatingSystem, email, password, language, uniqueClientId } => {
                Some(APIRequestBody::RegisterClient { operatingSystem: operatingSystem, email: email, password: password, language: language, uniqueClientId: uniqueClientId })
            },
            APIEndpoint::AccountStatus { .. } => {
                None
            },
            APIEndpoint::AccountDetails { .. } => {
                None
            },
            APIEndpoint::AccountKey { master, main, hmac, .. } => {
                Some(APIRequestBody::AccountKey { master: master, main: main, hmac: hmac })
            },
            APIEndpoint::ReadFolders { .. } => {
                None
            },
            APIEndpoint::CreateFolder { path, name, encrypted, .. } => {
                Some(APIRequestBody::CreateFolder { folderName: name, folderPath: path, encrypted: encrypted })
            },
            APIEndpoint::RegisterSyncSession { .. } => {
                None
            },
            APIEndpoint::FinishSyncSession { session_data, .. } => {
                Some(APIRequestBody::FinishSyncSession { session_data: session_data })
            },
            APIEndpoint::ReadSyncSession { .. } => {
                None
            },
            APIEndpoint::ReadSyncSessions { .. } => {
                None
            },
            APIEndpoint::CheckBlock { .. } => {
                None
            },
            APIEndpoint::WriteBlock { chunk_data, .. } => {
                Some(APIRequestBody::AddBlock { chunk_data: chunk_data })

            },
            APIEndpoint::ReadBlock { .. } => {
                None
            },

        }
    }
}

// SD API

pub fn register_client<S, T>(email: S, password: T) -> Result<(Token, UniqueClientID), SDAPIError> where S: Into<String>, T: Into<String> {

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
    let endpoint = APIEndpoint::RegisterClient{ operatingSystem: os, email: &em, password: &pa, language: "en-US", uniqueClientId: &uid };

    let client = reqwest::Client::new().unwrap();
    let request = client.post(&endpoint.url())
        .json(&endpoint.body().unwrap());

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
    let endpoint = APIEndpoint::AccountStatus { token: token };

    let client = reqwest::Client::new().unwrap();
    let request = client.get(&endpoint.url())
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
    let endpoint = APIEndpoint::AccountDetails { token: token };

    let client = reqwest::Client::new().unwrap();
    let request = client.get(&endpoint.url())
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

    let endpoint = APIEndpoint::AccountKey { token: token, master: &master, main: &main, hmac: &hmac };

    let client = reqwest::Client::new().unwrap();
    let request = client.get(&endpoint.url())
        .json(&endpoint.body().unwrap())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    let wrapped_keyset: WrappedKeyset = try!(serde_json::from_str(&response));

    Ok((wrapped_keyset.master, wrapped_keyset.main, wrapped_keyset.hmac))
}

pub fn read_folders(token: &Token) -> Result<Vec<RegisteredFolder>, SDAPIError> {

    let endpoint = APIEndpoint::ReadFolders { token: token };

    let client = reqwest::Client::new().unwrap();
    let request = client.get(&endpoint.url())
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

pub fn create_folder<S, T>(token: &Token, path: S, name: T, encrypted: bool) -> Result<i32, SDAPIError> where S: Into<String>, T: Into<String> {

    let pa = path.into();
    let na = name.into();

    let endpoint = APIEndpoint::CreateFolder { token: token, path: &pa, name: &na, encrypted: encrypted };

    let client = reqwest::Client::new().unwrap();
    let request = client.post(&endpoint.url())
        .json(&endpoint.body().unwrap())
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

// sync handling

pub fn register_sync_session<S>(token: &Token, folder_id: i32, name: S, encrypted: bool) -> Result<(), SDAPIError> where S: Into<String> {

    let na = name.into();

    let endpoint = APIEndpoint::RegisterSyncSession { token: token, folder_id: folder_id, name: &na, encrypted: encrypted };


    let client = reqwest::Client::new().unwrap();
    let request = client.post(&endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());
    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Created => {},
        _ => return Err(SDAPIError::RequestFailed)
    }

    Ok(())
}

pub fn finish_sync_session<S>(token: &Token, folder_id: i32, name: S, encrypted: bool, session_data: &[u8], size: usize) -> Result<(), SDAPIError> where S: Into<String> {

    let na = name.into();

    let endpoint = APIEndpoint::FinishSyncSession { token: token, folder_id: folder_id, name: &na, encrypted: encrypted, size: size, session_data: session_data };

    let (body, content_length, boundary) = multipart_for_bytes(session_data, &na);

    let client = reqwest::Client::new().unwrap();
    let request = client.post(&endpoint.url())
        .body(body)
        .header(SDAuthToken(token.token.to_owned()))
        .header(ContentType(boundary.to_owned()))
        .header(ContentLength(content_length));

    let result = try!(request.send());

    match result.status() {
        &reqwest::StatusCode::Ok => Ok(()),
        &reqwest::StatusCode::Created => Ok(()),
        _ => return Err(SDAPIError::RequestFailed)
    }
}


// block handling

pub fn check_block<S>(token: &Token, name: S) -> Result<bool, SDAPIError> where S: Into<String> {

    let na = name.into();

    let endpoint = APIEndpoint::CheckBlock { token: token, name: &na };

    let client = reqwest::Client::new().unwrap();

    let request = client.head(&endpoint.url()).header(SDAuthToken(token.token.to_owned()));

    let result = try!(request.send());

    match result.status() {
        &reqwest::StatusCode::Ok => Ok(true),
        &reqwest::StatusCode::NotFound => Ok(false),
        _ => return Err(SDAPIError::RequestFailed)
    }
}

pub fn write_block<S, T>(token: &Token, session: S, name: T, chunk_data: &Option<Vec<u8>>) -> Result<(), SDAPIError> where S: Into<String>, T: Into<String> {

    let na = name.into();
    let ses = session.into();

    let endpoint = APIEndpoint::WriteBlock { token: token, name: &na, session: &ses, chunk_data: chunk_data };


    let client = reqwest::Client::new().unwrap();
    let mut request = client.post(&endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));
    if let Some(ref data) = *chunk_data {
        let (body, content_length, boundary) = multipart_for_bytes(data, &na);

        request = request.body(body)
        .header(ContentType(boundary.to_owned()))
        .header(ContentLength(content_length));
    }
    let result = try!(request.send());

    match result.status() {
        &reqwest::StatusCode::Ok => Ok(()),
        &reqwest::StatusCode::Created => Ok(()),
        &reqwest::StatusCode::NotFound => Err(SDAPIError::RetryUpload),
        _ => return Err(SDAPIError::RequestFailed)
    }
}

pub fn read_block<'a, S>(token: &Token, name: &'a str) -> Result<Block<'a>, SDAPIError> where S: Into<String> {
    let endpoint = APIEndpoint::ReadBlock { token: token, name: name };

    let client = reqwest::Client::new().unwrap();
    let request = client.post(&endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::NotFound => return Err(SDAPIError::BlockMissing),
        _ => return Err(SDAPIError::RequestFailed)
    };
    let mut buffer = Vec::new();

    try!(result.read_to_end(&mut buffer));
    Ok(Block { name: name, chunk_data: buffer })
}

fn multipart_for_bytes(chunk_data: &[u8], name: &str) -> (Vec<u8>, usize, &'static str) {

    let mut body: Vec<u8> = Vec::new();

    // these are compile time optimizations
    let header_boundary: &'static str = "SAFEDRIVEBINARY";
    let rn: &'static [u8; 4] = br"\r\n";
    let body_boundary: &'static [u8; 17] = br"--SAFEDRIVEBINARY";
    let end_boundary: &'static [u8; 19] =  br"--SAFEDRIVEBINARY--";
    let content_type: &'static [u8; 38] = br"Content-Type: application/octet-stream";


    let disp = format!("Content-Disposition: form-data; name=\"{}\";", name);


    body.extend(body_boundary.as_ref());
    body.extend(rn);
    body.extend(disp.as_bytes());
    body.extend(rn);
    body.extend(content_type.as_ref());
    body.extend(rn);
    body.extend(rn);

    body.extend(chunk_data);
    body.extend(rn);
    body.extend(end_boundary.as_ref());

    let content_length = body.len();

    (body, content_length, header_boundary)
}

