#![allow(non_snake_case)]
#![allow(unused_variables)]

use std::io::Read;
use std::collections::HashMap;

/// external crate imports

use rustc_serialize::hex::ToHex;
use rand::distributions::{IndependentSample, Range};
use std::{thread, time};

/// internal imports

use error::SDAPIError;
use models::*;
use session::*;
use keys::*;
use constants::*;
use USER_AGENT;
use binformat::BinaryWriter;


header! { (UserAgent, "User-Agent") => [String] }
header! { (SDAuthToken, "SD-Auth-Token") => [String] }
header! { (ContentType, "Content-Type") => [String] }
header! { (ContentLength, "Content-Length") => [usize] }

#[allow(dead_code)]
struct ProgressReader<F> {
    inner: ::std::io::Cursor<Vec<u8>>,
    callback: F,
    content_length: u64,
    real_size: u64,
    current: u64,
    real_current: u64,
}

impl<F> ProgressReader<F> where F: FnMut(u64, u64, u64) + Send + Sync + 'static {
    #[allow(dead_code)]
    pub fn new(reader: Vec<u8>, callback: F, content_length: u64, real_size: u64) -> ProgressReader<F> {
        ProgressReader {
            inner: ::std::io::Cursor::new(reader),
            callback: callback,
            content_length: content_length,
            real_size: real_size,
            current: 0,
            real_current: 0,
        }
    }

    #[allow(dead_code)]
    pub fn to_body(self) -> ::reqwest::Body {
        let content_length = self.content_length;
        ::reqwest::Body::sized(self, content_length as u64)
    }
}

impl<F> Read for ProgressReader<F> where F: FnMut(u64, u64, u64) + Send + Sync + 'static {
    fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
        let r = self.inner.read(buf);

        match r {
            Ok(size) => {
                debug!("read {}", size);

                self.current += size as u64;

                let multiplier: f64 = self.current as f64 / self.content_length as f64;

                //let percentage: f64 = multiplier * 100.0;

                let real_updated: f64 = self.real_size as f64 * multiplier;

                let real_new = real_updated - self.real_current as f64;

                self.real_current = real_updated as u64;

                (self.callback)(self.real_size, self.real_current, real_new as u64);
            },
            Err(e) => {
                debug!("error reading: {}", e);
                return Err(e);
            },
        }

        r
    }
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum APIEndpoint<'a> {
    ErrorLog { operatingSystem: &'a str, clientVersion: &'a str, uniqueClientId: &'a str, description: &'a str, context: &'a str, log: &'a Vec<String> },
    RegisterClient { email: &'a str, password: &'a str, operatingSystem: &'a str, language: &'a str, uniqueClientId: &'a str },
    UnregisterClient,
    GetClients { email: &'a str, password: &'a str, },
    AccountStatus,
    AccountDetails,
    AccountKey { master: &'a str, main: &'a str, hmac: &'a str, tweak: &'a str },
    ReadFolders,
    CreateFolder { folderPath: &'a str, folderName: &'a str, encrypted: bool, syncing: bool },
    UpdateFolder { folderPath: &'a str, folderName: &'a str, syncing: bool, id: u64 },

    DeleteFolder { folder_id: u64 },
    RegisterSyncSession { folder_id: u64, name: &'a str, encrypted: bool },
    #[serde(skip_serializing)]
    FinishSyncSession { folder_id: u64, encrypted: bool, size: usize, session: &'a WrappedSyncSession },
    ReadSyncSession { name: &'a str, encrypted: bool },
    ReadSyncSessions { encrypted: bool },
    DeleteSyncSession { session_id: u64 },
    DeleteSyncSessions { timestamp: i64 },
    CheckBlock { name: &'a str },
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
            _ => {},
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
            APIEndpoint::UnregisterClient => {
                ::reqwest::Method::Post
            },
            APIEndpoint::GetClients { .. } => {
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
            APIEndpoint::UpdateFolder { .. } => {
                ::reqwest::Method::Put
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
            APIEndpoint::UnregisterClient => {
                format!("/api/1/client/unRegister")
            },
            APIEndpoint::GetClients { .. } => {
                format!("/api/1/client")
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
            APIEndpoint::UpdateFolder { .. } => {
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
pub fn report_error<'a>(clientVersion: &'a str, uniqueClientId: &'a str, operatingSystem: &'a str, description: &'a str, context: &'a str, log: &'a Vec<String>) -> Result<(), SDAPIError> {

    let endpoint = APIEndpoint::ErrorLog {
        operatingSystem: operatingSystem,
        uniqueClientId: uniqueClientId,
        clientVersion: clientVersion,
        description: description,
        context: context,
        log: log,
    };

    let user_agent = &**USER_AGENT.read();
    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .json(&endpoint);

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => return Ok(()),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }
}

pub fn register_client<'a>(operatingSystem: &str, languageCode: &str, uniqueClientId: &'a str, email: &'a str, password: &'a str) -> Result<Token, SDAPIError> {

    let endpoint = APIEndpoint::RegisterClient {
        operatingSystem: operatingSystem,
        email: email,
        password: password,
        language: languageCode,
        uniqueClientId: uniqueClientId,
    };

    let user_agent = &**USER_AGENT.read();
    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .json(&endpoint);

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let token: Token = ::serde_json::from_str(&response)?;
                return Ok(token);
            },
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }
}

pub fn unregister_client<'a>(token: &Token) -> Result<(), SDAPIError> {

    let endpoint = APIEndpoint::UnregisterClient;

    let user_agent = &**USER_AGENT.read();
    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => return Ok(()),
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }
}

pub fn list_clients(email: &str, password: &str) -> Result<Vec<SoftwareClient>, SDAPIError> {

    let endpoint = APIEndpoint::GetClients {
        email: email,
        password: password,
    };

    let user_agent = &**USER_AGENT.read();
    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .json(&endpoint);

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let clients: Vec<SoftwareClient> = ::serde_json::from_str(&response)?;
                return Ok(clients);
            },
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }
}

pub fn account_status(token: &Token) -> Result<AccountStatus, SDAPIError> {
    let endpoint = APIEndpoint::AccountStatus;

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let account_status: AccountStatus = ::serde_json::from_str(&response)?;
                return Ok(account_status);
            },
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }
}

#[allow(dead_code)]
pub fn account_details(token: &Token) -> Result<AccountDetails, SDAPIError> {
    let endpoint = APIEndpoint::AccountDetails;

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let account_details: AccountDetails = ::serde_json::from_str(&response)?;
                return Ok(account_details);
            },
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }

}

pub fn account_key(token: &Token, new_wrapped_keyset: &WrappedKeyset) -> Result<WrappedKeyset, SDAPIError> {

    let endpoint = APIEndpoint::AccountKey {
        master: &new_wrapped_keyset.master.to_hex(),
        main: &new_wrapped_keyset.main.to_hex(),
        hmac: &new_wrapped_keyset.hmac.to_hex(),
        tweak: &new_wrapped_keyset.tweak.to_hex(),
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .json(&endpoint)
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let wrapped_keyset_b: WrappedKeysetBody = ::serde_json::from_str(&response)?;
                return Ok(WrappedKeyset::from(wrapped_keyset_b));
            },
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }


}

pub fn read_folders(token: &Token) -> Result<Vec<RegisteredFolder>, SDAPIError> {

    let endpoint = APIEndpoint::ReadFolders;

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let folders: Vec<RegisteredFolder> = ::serde_json::from_str(&response)?;
                return Ok(folders);
            },
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}", result.status(), &response)))
        }
    }

}

pub fn create_folder(token: &Token, path: &str, name: &str, encrypted: bool) -> Result<u64, SDAPIError> {

    let endpoint = APIEndpoint::CreateFolder {
        folderPath: path,
        folderName: name,
        encrypted: encrypted,
        syncing: true,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .json(&endpoint)
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let folder_response: CreateFolderResponse = ::serde_json::from_str(&response)?;
                return Ok(folder_response.id);
            },
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }


}

pub fn update_folder(token: &Token, path: &str, name: &str, syncing: bool, uniqueID: u64) -> Result<(), SDAPIError> {

    let endpoint = APIEndpoint::UpdateFolder {
        folderPath: path,
        folderName: name,
        syncing: syncing,
        id: uniqueID,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .json(&endpoint)
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => return Ok(()),
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }


}

pub fn delete_folder(token: &Token, folder_id: u64) -> Result<(), SDAPIError> {
    let endpoint = APIEndpoint::DeleteFolder {
        folder_id: folder_id,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => return Ok(()),
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }

}

/// sync session handling

pub fn read_sessions(token: &Token) -> Result<HashMap<String, HashMap<u64, Vec<SyncSession>>>, SDAPIError> {

    let endpoint = APIEndpoint::ReadSyncSessions {
        encrypted: true,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let sessions: HashMap<String, HashMap<u64, Vec<SyncSession>>> = ::serde_json::from_str(&response)?;
                return Ok(sessions);
            },
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }


}

pub fn register_sync_session(token: &Token, folder_id: u64, name: &str, encrypted: bool) -> Result<(), SDAPIError> {

    let endpoint = APIEndpoint::RegisterSyncSession {
        folder_id: folder_id,
        name: name,
        encrypted: encrypted,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => return Ok(()),
            &::reqwest::StatusCode::Created => {},
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }

}

pub fn finish_sync_session<'a, F>(token: &Token, folder_id: u64, encrypted: bool, session: &[WrappedSyncSession], size: usize, progress: F) -> Result<(), SDAPIError> where F: FnMut(u64, u64, u64) + Send + Sync + 'static {

    let endpoint = APIEndpoint::FinishSyncSession {
        folder_id: folder_id,
        encrypted: encrypted,
        size: size,
        session: &session[0],
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {

        let (multipart_body, content_length, real_size) = multipart_for_binary(session, "file");
        
        let request = client.request(endpoint.method(), endpoint.url())
            .body(multipart_body)
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()))
            .header(ContentType(format!("multipart/form-data; boundary={}", MULTIPART_BOUNDARY.to_owned())))
            .header(ContentLength(content_length));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        match result.status() {
            &::reqwest::StatusCode::Ok => return Ok(()),
            &::reqwest::StatusCode::Created => return Ok(()),
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }
}

pub fn read_session<'a>(token: &Token, folder_id: u64, name: &'a str, encrypted: bool) -> Result<SyncSessionResponse<'a>, SDAPIError> {
    let endpoint = APIEndpoint::ReadSyncSession {
        name: name,
        encrypted: encrypted,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let mut buffer = Vec::new();

                trace!("reading data");
                result.read_to_end(&mut buffer)?;

                trace!("returning data");
                return Ok(SyncSessionResponse {
                              name: name,
                              chunk_data: buffer,
                              folder_id: folder_id,
                          });
            },
            &::reqwest::StatusCode::NotFound => return Err(SDAPIError::SessionMissing),
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let mut response = String::new();

                result.read_to_string(&mut response)?;

                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => return Err(SDAPIError::Internal(format!("unexpected status code: {}", result.status()))),
        };

    }

}

pub fn delete_session(token: &Token, session_id: u64) -> Result<(), SDAPIError> {
    let endpoint = APIEndpoint::DeleteSyncSession {
        session_id: session_id,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => return Ok(()),
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::NotFound => {},
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }

}

pub fn delete_sessions(token: &Token, timestamp: i64) -> Result<(), SDAPIError> {
    let endpoint = APIEndpoint::DeleteSyncSessions {
        timestamp: timestamp,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => return Ok(()),
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::NotFound => {},
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }

}

/// block handling
#[allow(dead_code)]
pub fn check_block(token: &Token, name: &str) -> Result<bool, SDAPIError> {

    let endpoint = APIEndpoint::CheckBlock {
        name: name,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        trace!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok => return Ok(true),
            &::reqwest::StatusCode::NotFound => return Ok(false),
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }
}

#[allow(dead_code)]
pub fn write_blocks<F, T>(token: &Token, session: &str, blocks: &[T], progress: F) -> Result<Vec<String>, SDAPIError> where F: FnMut(u64, u64, u64) + Send + Sync + 'static, T: ::binformat::BinaryWriter {

    let endpoint = APIEndpoint::WriteBlocks {
        session: session,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let (multipart_body, content_length, real_size) = multipart_for_binary(blocks, "files");
        
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()))
            .header(ContentType(format!("multipart/form-data; boundary={}", MULTIPART_BOUNDARY.to_owned())))
            .header(ContentLength(content_length))
            .body(multipart_body);

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");
        let mut response = String::new();
        trace!("reading response");
        result.read_to_string(&mut response)?;

        debug!("response: {}", response);

        match result.status() {
            &::reqwest::StatusCode::Ok | &::reqwest::StatusCode::Created => {
                let missing: Vec<String> = ::serde_json::from_str(&response)?;
                return Ok(missing);
            },
            &::reqwest::StatusCode::BadRequest => {
                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => {
                return Err(SDAPIError::Internal(format!("unexpected response(HTTP{}): {}",
                                                        result.status(),
                                                        &response)))
            },
        }
    }
}


pub fn read_block(token: &Token, name: &str) -> Result<Vec<u8>, SDAPIError> {
    let endpoint = APIEndpoint::ReadBlock {
        name: name,
    };

    let user_agent = &**USER_AGENT.read();

    let client = ::reqwest::Client::new().unwrap();

    let retries: u8 = 3;
    let mut retries_left: u8 = retries;

    loop {
        let request = client.request(endpoint.method(), endpoint.url())
            .header(::reqwest::header::Connection::close())
            .header(UserAgent(user_agent.to_string()))
            .header(SDAuthToken(token.token.to_owned()));

        let failed_count = retries - retries_left;
        let mut rng = ::rand::thread_rng();
        let backoff_multiplier = Range::new(0.0, 1.5).ind_sample(&mut rng);

        if failed_count >= 1 {
            let backoff_time = backoff_multiplier * (failed_count as f64 * failed_count as f64);
            let delay = time::Duration::from_millis((backoff_time * 1000.0) as u64);
            thread::sleep(delay);
        }

        trace!("sending request");
        let mut result = request.send()?;
        trace!("response received");

        match result.status() {
            &::reqwest::StatusCode::Ok => {
                let mut buffer = Vec::new();
                trace!("reading data");
                result.read_to_end(&mut buffer)?;
                trace!("returning data");

                return Ok(buffer);
            },
            &::reqwest::StatusCode::NotFound => return Err(SDAPIError::BlockMissing),
            &::reqwest::StatusCode::Unauthorized => return Err(SDAPIError::Authentication),
            &::reqwest::StatusCode::BadRequest => {
                let mut response = String::new();

                result.read_to_string(&mut response)?;

                let error: ServerErrorResponse = ::serde_json::from_str(&response)?;
                return Err(SDAPIError::Internal(error.message));
            },
            &::reqwest::StatusCode::ServiceUnavailable => {
                retries_left = retries_left - 1;
                if retries_left <= 0 {
                    return Err(SDAPIError::ServiceUnavailable);
                }
            },
            _ => return Err(SDAPIError::Internal(format!("unexpected status code: {}", result.status()))),
        };
    }

}

fn multipart_for_binary<T>(items: &[T], field_name: &str) -> (Vec<u8>, usize, usize) where T: ::binformat::BinaryWriter {
    let mut body = Vec::new();
    /// these are compile time optimizations
    let rn = b"\r\n";
    let body_boundary = br"--SAFEDRIVEBINARY";
    let end_boundary =  br"--SAFEDRIVEBINARY--";
    let enc = br"Content-Transfer-Encoding: binary";

    body.extend(rn);
    body.extend(rn);

    let mut real_size = 0;

    for item in items {
        let name = item.name();
        let data = item.as_binary();
        let size = item.len();
        real_size += size;
        let disp = format!("Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"", field_name, name);
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

    (body, content_length, real_size)
}
