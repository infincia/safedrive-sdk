#![allow(non_snake_case)]

use std::io::Read;
use std::collections::HashMap;

/// external crate imports

use ::rustc_serialize::hex::{ToHex};

/// internal imports

use ::error::SDAPIError;
use ::models::*;
use ::block::*;
use ::session::*;
use ::keys::*;
use ::constants::*;
use ::USER_AGENT;
use ::binformat::BinaryWriter;


header! { (UserAgent, "User-Agent") => [String] }
header! { (SDAuthToken, "SD-Auth-Token") => [String] }
header! { (ContentType, "Content-Type") => [String] }
header! { (ContentLength, "Content-Length") => [usize] }

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum APIEndpoint<'a> {
    ErrorLog { operatingSystem: &'a str, clientVersion: &'a str, uniqueClientId: &'a str, description: &'a str, context: &'a str, log: &'a [&'a str] },
    RegisterClient { email: &'a str, password: &'a str, operatingSystem: &'a str, language: &'a str, uniqueClientId: &'a str },
    AccountStatus,
    AccountDetails,
    AccountKey { master: &'a str, main: &'a str, hmac: &'a str, tweak: &'a str },
    ReadFolders,
    CreateFolder { folderPath: &'a str, folderName: &'a str, encrypted: bool },
    DeleteFolder { folder_id: u64 },
    RegisterSyncSession { folder_id: u64, name: &'a str, encrypted: bool },
    #[serde(skip_serializing)]
    FinishSyncSession { folder_id: u64, encrypted: bool, size: usize, session: &'a WrappedSyncSession },
    ReadSyncSession { name: &'a str, encrypted: bool },
    ReadSyncSessions { encrypted: bool },
    DeleteSyncSession { session_id: u64 },
    DeleteSyncSessions { timestamp: i64 },
    CheckBlock { name: &'a str },
    WriteBlock { session: &'a str, name: &'a str },
    WriteBlocks { session: &'a str },
    ReadBlock { name: &'a str },
}

impl<'a> APIEndpoint<'a> {

    pub fn url(&self) -> ::reqwest::Url {
        let mut base = String::new();
        base += &self.protocol();
        base += &self.domain();
        let url_base = ::reqwest::Url::parse(&base).unwrap();
        let mut url = url_base.join(&self.path()).unwrap();
        match *self {
            APIEndpoint::DeleteFolder { folder_id, .. } => {
                url.query_pairs_mut()
                    .clear()
                    .append_pair("folderIds", &format!("{}", folder_id));
            },
            APIEndpoint::DeleteSyncSession { session_id, .. } => {
                url.query_pairs_mut()
                    .clear()
                    .append_pair("sessionIds", &format!("{}", session_id));
            },
            APIEndpoint::DeleteSyncSessions { timestamp, .. } => {
                url.query_pairs_mut()
                    .clear()
                    .append_pair("date", &format!("{}", timestamp));
            },
            _ => {}
        }
        trace!("constructed url request: {}", url);
        url
    }

    pub fn domain(&self) -> String {
        api_domain().to_string()
    }

    pub fn protocol(&self) -> String {
        "https://".to_string()
    }

    pub fn method(&self) -> ::reqwest::Method {
        match *self {
            APIEndpoint::ErrorLog { .. } => {
                ::reqwest::Method::Post
            },
            APIEndpoint::RegisterClient { .. } => {
                ::reqwest::Method::Post
            },
            APIEndpoint::AccountStatus => {
                ::reqwest::Method::Get
            },
            APIEndpoint::AccountDetails => {
                ::reqwest::Method::Get
            },
            APIEndpoint::AccountKey { .. } => {
                ::reqwest::Method::Post
            },
            APIEndpoint::ReadFolders => {
                ::reqwest::Method::Get
            },
            APIEndpoint::CreateFolder { .. } => {
                ::reqwest::Method::Post
            },
            APIEndpoint::DeleteFolder { .. } => {
                ::reqwest::Method::Delete
            },
            APIEndpoint::RegisterSyncSession { .. } => {
                ::reqwest::Method::Post
            },
            APIEndpoint::FinishSyncSession { .. } => {
                ::reqwest::Method::Post
            },
            APIEndpoint::ReadSyncSession { .. } => {
                ::reqwest::Method::Get
            },
            APIEndpoint::ReadSyncSessions { .. } => {
                ::reqwest::Method::Get
            },
            APIEndpoint::DeleteSyncSession { .. } => {
                ::reqwest::Method::Delete
            },
            APIEndpoint::DeleteSyncSessions { .. } => {
                ::reqwest::Method::Delete
            },
            APIEndpoint::CheckBlock { .. } => {
                ::reqwest::Method::Head
            },
            APIEndpoint::WriteBlock { .. } => {
                ::reqwest::Method::Post
            },
            APIEndpoint::WriteBlocks { .. } => {
                ::reqwest::Method::Post
            },
            APIEndpoint::ReadBlock { .. } => {
                ::reqwest::Method::Get
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
            APIEndpoint::AccountStatus => {
                format!("/api/1/account/status")
            },
            APIEndpoint::AccountDetails => {
                format!("/api/1/account/details")
            },
            APIEndpoint::AccountKey { .. } => {
                format!("/api/1/account/key")
            },
            APIEndpoint::ReadFolders => {
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
            APIEndpoint::FinishSyncSession { size, session, .. } => {
                format!("/api/1/sync/session/{}/{}", session.name(), size)
            },
            APIEndpoint::ReadSyncSession { name, .. } => {
                format!("/api/1/sync/session/{}", name)
            },
            APIEndpoint::ReadSyncSessions { .. } => {
                format!("/api/1/sync/session")
            },
            APIEndpoint::DeleteSyncSession { .. } => {
                format!("/api/1/sync/session/ids")
            },
            APIEndpoint::DeleteSyncSessions { .. } => {
                format!("/api/1/sync/session/date")
            },
            APIEndpoint::CheckBlock { name, .. } => {
                format!("/api/1/sync/block/{}", name)
            },
            APIEndpoint::WriteBlock { session, name, .. } => {
                format!("/api/1/sync/block/{}/{}", name, session)
            },
            APIEndpoint::WriteBlocks { session } => {
                format!("/api/1/sync/blocks/multi/{}", session)
            },
            APIEndpoint::ReadBlock { name, .. } => {
                format!("/api/1/sync/block/{}", name)
            },

        };

        path
    }
}

/// SD API
#[allow(dead_code)]
pub fn report_error<'a>(clientVersion: &'a str, uniqueClientId: &'a str, operatingSystem: &'a str, description: &'a str, context: &'a str, log: &'a [&'a str]) -> Result<(), SDAPIError> {

    let endpoint = APIEndpoint::ErrorLog { operatingSystem: operatingSystem, uniqueClientId: uniqueClientId, clientVersion: clientVersion, description: description, context: context, log: log };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .json(&endpoint);

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    Ok(())
}

pub fn register_client<'a>(operatingSystem: &str, languageCode: &str, uniqueClientId: &'a str, email: &'a str, password: &'a str) -> Result<Token, SDAPIError> {

    let endpoint = APIEndpoint::RegisterClient{ operatingSystem: operatingSystem, email: email, password: password, language: languageCode, uniqueClientId: uniqueClientId };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .json(&endpoint);

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }


    let token: Token = try!(::serde_json::from_str(&response));

    Ok(token)
}

pub fn account_status(token: &Token) -> Result<AccountStatus, SDAPIError> {
    let endpoint = APIEndpoint::AccountStatus;

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }



    let account_status: AccountStatus = try!(::serde_json::from_str(&response));

    Ok(account_status)
}

#[allow(dead_code)]
pub fn account_details(token: &Token) -> Result<AccountDetails, SDAPIError> {
    let endpoint = APIEndpoint::AccountDetails;

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let account_details: AccountDetails = try!(::serde_json::from_str(&response));

    Ok(account_details)
}

pub fn account_key(token: &Token, new_wrapped_keyset: &WrappedKeyset) -> Result<WrappedKeyset, SDAPIError> {

    let endpoint = APIEndpoint::AccountKey { master: &new_wrapped_keyset.master.to_hex(), main: &new_wrapped_keyset.main.to_hex(), hmac: &new_wrapped_keyset.hmac.to_hex(), tweak: &new_wrapped_keyset.tweak.to_hex() };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .json(&endpoint)
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let wrapped_keyset_b: WrappedKeysetBody = try!(::serde_json::from_str(&response));

    Ok(WrappedKeyset::from(wrapped_keyset_b))
}

pub fn read_folders(token: &Token) -> Result<Vec<RegisteredFolder>, SDAPIError> {

    let endpoint = APIEndpoint::ReadFolders;

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let folders: Vec<RegisteredFolder> = try!(::serde_json::from_str(&response));

    Ok(folders)
}

pub fn create_folder<'a>(token: &Token, path: &'a str, name: &'a str, encrypted: bool) -> Result<u64, SDAPIError> {

    let endpoint = APIEndpoint::CreateFolder { folderPath: path, folderName: name, encrypted: encrypted };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .json(&endpoint)
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let folder_response: CreateFolderResponse = try!(::serde_json::from_str(&response));

    Ok(folder_response.id)
}

pub fn delete_folder(token: &Token, folder_id: u64) -> Result<(), SDAPIError> {
    let endpoint = APIEndpoint::DeleteFolder { folder_id: folder_id };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    Ok(())
}

/// sync session handling

pub fn read_sessions(token: &Token) -> Result<HashMap<String, HashMap<u64, Vec<SyncSession>>>, SDAPIError> {

    let endpoint = APIEndpoint::ReadSyncSessions { encrypted: true };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    let sessions: HashMap<String, HashMap<u64, Vec<SyncSession>>> = try!(::serde_json::from_str(&response));

    Ok(sessions)
}

pub fn register_sync_session<'a>(token: &Token, folder_id: u64, name: &'a str, encrypted: bool) -> Result<(), SDAPIError> {

    let endpoint = APIEndpoint::RegisterSyncSession { folder_id: folder_id, name: name, encrypted: encrypted };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Created => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    Ok(())
}

pub fn finish_sync_session<'a>(token: &Token, folder_id: u64, encrypted: bool, session: &[WrappedSyncSession], size: usize) -> Result<(), SDAPIError> {

    let endpoint = APIEndpoint::FinishSyncSession { folder_id: folder_id, encrypted: encrypted, size: size, session: &session[0] };

    let (multipart_body, content_length) = multipart_for_binary(session);

    trace!("session body: {}", String::from_utf8_lossy(&multipart_body));

    //trace!("body: {}", String::from_utf8_lossy(&body));

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .body(multipart_body)
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()))
        .header(ContentType(format!("multipart/form-data; boundary={}", MULTIPART_BOUNDARY.to_owned())))
        .header(ContentLength(content_length));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    match result.status() {
        &::reqwest::StatusCode::Ok => Ok(()),
        &::reqwest::StatusCode::Created => Ok(()),
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }
}

pub fn read_session<'a>(token: &Token, folder_id: u64, name: &'a str, encrypted: bool) -> Result<SyncSessionResponse<'a>, SDAPIError> {
    let endpoint = APIEndpoint::ReadSyncSession { name: name, encrypted: encrypted };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::NotFound => return Err(SDAPIError::SessionMissing),
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let mut response = String::new();

            try!(result.read_to_string(&mut response));

            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected status code: {}", result.status())))
    };

    let mut buffer = Vec::new();

    trace!("reading data");
    try!(result.read_to_end(&mut buffer));

    trace!("returning data");
    Ok(SyncSessionResponse { name: name, chunk_data: buffer, folder_id: folder_id })
}

pub fn delete_session(token: &Token, session_id: u64) -> Result<(), SDAPIError> {
    let endpoint = APIEndpoint::DeleteSyncSession { session_id: session_id };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::NotFound => {},
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    Ok(())
}

pub fn delete_sessions(token: &Token, timestamp: i64) -> Result<(), SDAPIError> {
    let endpoint = APIEndpoint::DeleteSyncSessions { timestamp: timestamp };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::NotFound => {},
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }

    Ok(())
}

/// block handling
#[allow(dead_code)]
pub fn check_block<'a>(token: &Token, name: &'a str) -> Result<bool, SDAPIError> {

    let endpoint = APIEndpoint::CheckBlock { name: name };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => Ok(true),
        &::reqwest::StatusCode::NotFound => Ok(false),
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }
}

pub fn write_block(token: &Token, session: &str, name: &str, block: &WrappedBlock, should_upload: bool) -> Result<(), SDAPIError> {

    let endpoint = APIEndpoint::WriteBlock { name: name, session: session };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let mut request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));
    if should_upload {
        let (body, content_length) = multipart_for_bytes(&block.to_binary(), name, true, true);
        //trace!("body: {}", String::from_utf8_lossy(&body));

        request = request.body(body)
        .header(ContentType(format!("multipart/form-data; boundary={}", MULTIPART_BOUNDARY.to_owned())))
        .header(ContentLength(content_length));
    }

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => Ok(()),
        &::reqwest::StatusCode::Created => Ok(()),
        &::reqwest::StatusCode::BadRequest => Err(SDAPIError::RetryUpload),
        &::reqwest::StatusCode::NotFound => Err(SDAPIError::RetryUpload),
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }
}

#[allow(dead_code)]
pub fn write_blocks(token: &Token, session: &str, name: &str, blocks: &[WrappedBlock], should_upload: bool) -> Result<(), SDAPIError> {

    let endpoint = APIEndpoint::WriteBlocks { session: session };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let mut request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));
    if should_upload {
        let mut multipart_body = Vec::new();
        let mut total_size = 0 as usize;
        for (i, block) in blocks.iter().enumerate() {
            let first = i == 0;
            let last = i == blocks.len() -1;
            let (body, content_length) = multipart_for_bytes(&block.to_binary(), name, first, last);
            total_size += content_length;
            multipart_body.extend(body);
        }
        //trace!("body: {}", String::from_utf8_lossy(&body));

        request = request.body(multipart_body)
            .header(ContentType(format!("multipart/form-data; boundary={}", MULTIPART_BOUNDARY.to_owned())))
            .header(ContentLength(total_size));
    }

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");
    let mut response = String::new();
    trace!("reading response");
    try!(result.read_to_string(&mut response));

    trace!("response: {}", response);

    match result.status() {
        &::reqwest::StatusCode::Ok => Ok(()),
        &::reqwest::StatusCode::Created => Ok(()),
        &::reqwest::StatusCode::BadRequest => Err(SDAPIError::RetryUpload),
        &::reqwest::StatusCode::NotFound => Err(SDAPIError::RetryUpload),
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
    }
}


pub fn read_block<'a>(token: &Token, name: &'a str) -> Result<Vec<u8>, SDAPIError> {
    let endpoint = APIEndpoint::ReadBlock { name: name };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();
    let request = client.request(endpoint.method(), endpoint.url())
        .header(::reqwest::header::Connection::close())
        .header(UserAgent(user_agent.to_string()))
        .header(SDAuthToken(token.token.to_owned()));

    trace!("sending request");
    let mut result = try!(request.send());
    trace!("response received");

    match result.status() {
        &::reqwest::StatusCode::Ok => {},
        &::reqwest::StatusCode::NotFound => return Err(SDAPIError::BlockMissing),
        &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
        &::reqwest::StatusCode::BadRequest => {
            let mut response = String::new();

            try!(result.read_to_string(&mut response));

            let error: ServerErrorResponse = try!(::serde_json::from_str(&response));
            return Err(SDAPIError::Internal(error.message))
        },
        &::reqwest::StatusCode::ServiceUnavailable => return Err(SDAPIError::ServiceUnavailable),

        _ => return Err(SDAPIError::Internal(format!("unexpected status code: {}", result.status())))
    };

    let mut buffer = Vec::new();

    trace!("reading data");

    try!(result.read_to_end(&mut buffer));

    trace!("returning data");

    Ok(buffer)
}

fn multipart_for_bytes(chunk_data: &[u8], name: &str) -> (Vec<u8>, usize) {

    let mut body: Vec<u8> = Vec::new();

    /// these are compile time optimizations
    let rn = b"\r\n";
    let body_boundary = br"--SAFEDRIVEBINARY";
    let end_boundary =  br"--SAFEDRIVEBINARY--";
    let content_type = br"Content-Type: application/octet-stream";


    let disp = format!("content-disposition: form-data; name=\"file\"; filename=\"{}\"", name);
    let enc = br"Content-Transfer-Encoding: binary";


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

    (body, content_length)
}

fn multipart_for_binary<T>(items: &[T]) -> (Vec<u8>, usize) where T: ::binformat::BinaryWriter {
    let mut body = Vec::new();
    /// these are compile time optimizations
    let rn = b"\r\n";
    let body_boundary = br"--SAFEDRIVEBINARY";
    let end_boundary =  br"--SAFEDRIVEBINARY--";
    let enc = br"Content-Transfer-Encoding: binary";

    body.extend(rn);
    body.extend(rn);

    for item in items {
        let name = item.name();
        let data = item.as_binary();
        let disp = format!("Content-Disposition: form-data; name=\"files\"; filename=\"{}\"", name);
        let content_type = br"Content-Type: application/octet-stream";

        body.extend(body_boundary.as_ref());
        body.extend(rn);
        body.extend(disp.as_bytes());
        body.extend(rn);
        body.extend(content_type.as_ref());
        body.extend(rn);
        body.extend(enc.as_ref());
        body.extend(rn);
        body.extend(rn);
        if item.should_include_data() {
            body.extend(data.as_slice());
        } else {
            body.extend(&[]);
        }
        body.extend(rn);
    }
    body.extend(end_boundary.as_ref());
    body.extend(rn);
    body.extend(rn);

    let content_length = body.len();

    (body, content_length)
}