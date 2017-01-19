#![allow(non_snake_case)]

use std::io::Read;

use std::collections::HashMap;

extern crate rustc_serialize;

use self::rustc_serialize::hex::{ToHex};

extern crate reqwest;
extern crate serde;
extern crate serde_json;

header! { (SDAuthToken, "SD-Auth-Token") => [String] }
header! { (ContentType, "Content-Type") => [String] }
header! { (ContentLength, "Content-Length") => [usize] }

use util::*;
use error::SDAPIError;
use models::*;
use keys::*;
use constants::*;
use CONFIGURATION;

pub enum APIEndpoint<'a> {
    ErrorLog { operatingSystem: &'a str, clientVersion: &'a str, uniqueClientId: &'a str, description: &'a str, context: &'a str, log: &'a [&'a str] },

    RegisterClient { email: &'a str, password: &'a str, operatingSystem: &'a str, language: &'a str, uniqueClientId: &'a str },
    AccountStatus { token: &'a Token },
    AccountDetails { token: &'a Token },
    AccountKey { token: &'a Token, master: &'a str, main: &'a str, hmac: &'a str, tweak: &'a str },
    ReadFolders { token: &'a Token },
    CreateFolder { token: &'a Token, path: &'a str, name: &'a str, encrypted: bool },
    DeleteFolder { token: &'a Token, folder_id: u32 },
    RegisterSyncSession { token: &'a Token, folder_id: u32, name: &'a str, encrypted: bool },
    FinishSyncSession { token: &'a Token, folder_id: u32, name: &'a str, encrypted: bool, size: usize, session_data: &'a [u8] },
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
        let c = CONFIGURATION.read().unwrap();
        match *c {
            Configuration::Staging => SDAPIDOMAIN_STAGING.to_string(),
            Configuration::Production => SDAPIDOMAIN_PRODUCTION.to_string(),
        }
    }

    pub fn protocol(&self) -> String {
        "https://".to_string()
    }

    pub fn method(&self) -> self::reqwest::Method {
        match *self {
            APIEndpoint::ErrorLog { .. } => {
                self::reqwest::Method::Post
            },
            APIEndpoint::RegisterClient { .. } => {
                self::reqwest::Method::Post
            },
            APIEndpoint::AccountStatus { .. } => {
                self::reqwest::Method::Get
            },
            APIEndpoint::AccountDetails { .. } => {
                self::reqwest::Method::Get
            },
            APIEndpoint::AccountKey { .. } => {
                self::reqwest::Method::Post
            },
            APIEndpoint::ReadFolders { .. } => {
                self::reqwest::Method::Get
            },
            APIEndpoint::CreateFolder { .. } => {
                self::reqwest::Method::Post
            },
            APIEndpoint::DeleteFolder { .. } => {
                self::reqwest::Method::Delete
            },
            APIEndpoint::RegisterSyncSession { .. } => {
                self::reqwest::Method::Post
            },
            APIEndpoint::FinishSyncSession { .. } => {
                self::reqwest::Method::Post
            },
            APIEndpoint::ReadSyncSession { .. } => {
                self::reqwest::Method::Get
            },
            APIEndpoint::ReadSyncSessions { .. } => {
                self::reqwest::Method::Get
            },
            APIEndpoint::CheckBlock { .. } => {
                self::reqwest::Method::Head
            },
            APIEndpoint::WriteBlock { .. } => {
                self::reqwest::Method::Post
            },
            APIEndpoint::ReadBlock { .. } => {
                self::reqwest::Method::Get
            },
        }
    }

    pub fn path(&self) -> String {
        let path = match *self {
            APIEndpoint::ErrorLog { .. } => {
                format!("/api/1/error/log")
            },
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
            APIEndpoint::DeleteFolder { .. } => {
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
}

// SD API

pub fn report_error<'a>(clientVersion: &'a str, uniqueClientId: &'a str, description: &'a str, context: &'a str, log: &'a [&'a str]) -> Result<(), SDAPIError> {

    let operatingSystem = get_current_os();

    let endpoint = APIEndpoint::ErrorLog { operatingSystem: operatingSystem, uniqueClientId: uniqueClientId, clientVersion: clientVersion, description: description, context: context, log: log };
    let body = ErrorLogBody { operatingSystem: operatingSystem, uniqueClientId: uniqueClientId, clientVersion: clientVersion, description: description, context: context, log: log };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .json(&body);

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    Ok(())
}

pub fn register_client<'a>(uniqueClientId: &'a str, email: &'a str, password: &'a str) -> Result<(Token, UniqueClientID), SDAPIError> {

    let operatingSystem = get_current_os();

    let endpoint = APIEndpoint::RegisterClient{ operatingSystem: operatingSystem, email: email, password: password, language: "en_US", uniqueClientId: uniqueClientId };
    let body = RegisterClientBody { operatingSystem: operatingSystem, email: email, password: password, language: "en_US", uniqueClientId: uniqueClientId };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .json(&body);

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }


    let token: Token = try!(serde_json::from_str(&response));

    let u = UniqueClientID { id: uniqueClientId.to_owned() };

    Ok((token, u))
}

pub fn account_status(token: &Token) -> Result<AccountStatus, SDAPIError> {
    let endpoint = APIEndpoint::AccountStatus { token: token };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }



    let account_status: AccountStatus = try!(serde_json::from_str(&response));

    Ok(account_status)
}

pub fn account_details(token: &Token) -> Result<AccountDetails, SDAPIError> {
    let endpoint = APIEndpoint::AccountDetails { token: token };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let account_details: AccountDetails = try!(serde_json::from_str(&response));

    Ok(account_details)
}

pub fn account_key(token: &Token, new_wrapped_keyset: &WrappedKeyset) -> Result<WrappedKeyset, SDAPIError> {

    let endpoint = APIEndpoint::AccountKey { token: token, master: &new_wrapped_keyset.master.to_hex(), main: &new_wrapped_keyset.main.to_hex(), hmac: &new_wrapped_keyset.hmac.to_hex(), tweak: &new_wrapped_keyset.tweak.to_hex() };
    let body = AccountKey { master: &new_wrapped_keyset.master.to_hex(), main: &new_wrapped_keyset.main.to_hex(), hmac: &new_wrapped_keyset.hmac.to_hex(), tweak: &new_wrapped_keyset.tweak.to_hex() };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .json(&body)
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let wrapped_keyset_b: WrappedKeysetBody = try!(serde_json::from_str(&response));

    Ok(WrappedKeyset::from(wrapped_keyset_b))
}

pub fn read_folders(token: &Token) -> Result<Vec<RegisteredFolder>, SDAPIError> {

    let endpoint = APIEndpoint::ReadFolders { token: token };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let folders: Vec<RegisteredFolder> = try!(serde_json::from_str(&response));

    Ok(folders)
}

pub fn create_folder<S, T>(token: &Token, path: S, name: T, encrypted: bool) -> Result<u32, SDAPIError> where S: Into<String>, T: Into<String> {

    let pa = path.into();
    let na = name.into();

    let endpoint = APIEndpoint::CreateFolder { token: token, path: &pa, name: &na, encrypted: encrypted };
    let body = CreateFolder { folderName: &na, folderPath: &pa, encrypted: encrypted };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .json(&body)
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let folder_response: CreateFolderResponse = try!(serde_json::from_str(&response));

    Ok(folder_response.id)
}

pub fn delete_folder(token: &Token, folder_id: u32) -> Result<(), SDAPIError> {
    let endpoint = APIEndpoint::DeleteFolder { token: token, folder_id: folder_id };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    Ok(())
}

// sync session handling

pub fn read_sessions(token: &Token) -> Result<HashMap<u32, Vec<SyncSession>>, SDAPIError> {

    let endpoint = APIEndpoint::ReadSyncSessions { token: token, encrypted: true };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let sessions: HashMap<u32, Vec<SyncSession>> = try!(serde_json::from_str(&response));

    Ok(sessions)
}

pub fn register_sync_session<S>(token: &Token, folder_id: u32, name: S, encrypted: bool) -> Result<(), SDAPIError> where S: Into<String> {

    let na = name.into();

    let endpoint = APIEndpoint::RegisterSyncSession { token: token, folder_id: folder_id, name: &na, encrypted: encrypted };


    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::Created => {},
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    Ok(())
}

pub fn finish_sync_session<S>(token: &Token, folder_id: u32, name: S, encrypted: bool, session_data: &[u8], size: usize) -> Result<(), SDAPIError> where S: Into<String> {

    let na = name.into();

    let endpoint = APIEndpoint::FinishSyncSession { token: token, folder_id: folder_id, name: &na, encrypted: encrypted, size: size, session_data: session_data };

    let (body, content_length, boundary) = multipart_for_bytes(session_data, &na);

    debug!("body: {}", String::from_utf8_lossy(&body));

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .body(body)
        .header(SDAuthToken(token.token.to_owned()))
        .header(ContentType(format!("multipart/form-data; boundary={}", boundary.to_owned())))
        .header(ContentLength(content_length));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => Ok(()),
        &reqwest::StatusCode::Created => Ok(()),
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }
}

pub fn read_session<'a>(token: &Token, name: &'a str, encrypted: bool) -> Result<SyncSessionData<'a>, SDAPIError> {
    let endpoint = APIEndpoint::ReadSyncSession { token: token, name: name, encrypted: encrypted };


    let client = reqwest::Client::new().unwrap();

    let request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::NotFound => return Err(SDAPIError::SessionMissing),
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected status code: {}", result.status())))
    };
    let mut buffer = Vec::new();

    try!(result.read_to_end(&mut buffer));

    Ok(SyncSessionData { name: name, chunk_data: buffer })
}


// block handling

pub fn check_block<S>(token: &Token, name: S) -> Result<bool, SDAPIError> where S: Into<String> {

    let na = name.into();

    let endpoint = APIEndpoint::CheckBlock { token: token, name: &na };

    let client = reqwest::Client::new().unwrap();

    let request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => Ok(true),
        &reqwest::StatusCode::NotFound => Ok(false),
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }
}

pub fn write_block<S, T>(token: &Token, session: S, name: T, chunk_data: &Option<Vec<u8>>) -> Result<(), SDAPIError> where S: Into<String>, T: Into<String> {

    let na = name.into();
    let ses = session.into();

    let endpoint = APIEndpoint::WriteBlock { token: token, name: &na, session: &ses, chunk_data: chunk_data };


    let client = reqwest::Client::new().unwrap();
    let mut request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));
    if let Some(ref data) = *chunk_data {
        let (body, content_length, boundary) = multipart_for_bytes(data, &na);
        //debug!("body: {}", String::from_utf8_lossy(&body));

        request = request.body(body)
        .header(ContentType(format!("multipart/form-data; boundary={}", boundary.to_owned())))
        .header(ContentLength(content_length));
    }
    let mut result = try!(request.send());

    let mut response = String::new();

    try!(result.read_to_string(&mut response));

    debug!("response: {}", response);

    match result.status() {
        &reqwest::StatusCode::Ok => Ok(()),
        &reqwest::StatusCode::Created => Ok(()),
        &reqwest::StatusCode::BadRequest => Err(SDAPIError::RetryUpload),
        &reqwest::StatusCode::NotFound => Err(SDAPIError::RetryUpload),
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }
}

pub fn read_block<'a, S>(token: &Token, name: &'a str) -> Result<Block<'a>, SDAPIError> where S: Into<String> {
    let endpoint = APIEndpoint::ReadBlock { token: token, name: name };

    let client = reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), &endpoint.url())
        .header(SDAuthToken(token.token.to_owned()));

    let mut result = try!(request.send());

    match result.status() {
        &reqwest::StatusCode::Ok => {},
        &reqwest::StatusCode::NotFound => return Err(SDAPIError::BlockMissing),
        &reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),

        _ => return Err(SDAPIError::Internal(format!("unexpected status code: {}", result.status())))
    };
    let mut buffer = Vec::new();

    try!(result.read_to_end(&mut buffer));

    Ok(Block { name: name, chunk_data: buffer })
}

fn multipart_for_bytes(chunk_data: &[u8], name: &str) -> (Vec<u8>, usize, &'static str) {

    let mut body: Vec<u8> = Vec::new();

    // these are compile time optimizations
    let header_boundary: &'static str = "-----SAFEDRIVEBINARY";
    let rn: &'static [u8; 2] = b"\r\n";
    let body_boundary: &'static [u8; 22] = br"-------SAFEDRIVEBINARY";
    let end_boundary: &'static [u8; 24] =  br"-------SAFEDRIVEBINARY--";
    let content_type: &'static [u8; 38] = br"Content-Type: application/octet-stream";


    let disp = format!("content-disposition: form-data; name=file; filename={}", name);
    let enc: &'static [u8; 33] = br"Content-Transfer-Encoding: binary";

    body.extend(rn);
    body.extend(rn);
    body.extend(body_boundary.as_ref());
    body.extend(rn);

    body.extend(disp.as_bytes());
    body.extend(rn);

    body.extend(content_type.as_ref());
    body.extend(rn);

    body.extend(enc.as_ref());
    body.extend(rn);
    body.extend(rn);

    body.extend(chunk_data);
    body.extend(rn);

    body.extend(end_boundary.as_ref());
    body.extend(rn);
    body.extend(rn);

    let content_length = body.len();

    (body, content_length, header_boundary)
}

