use std::collections::hash_map::HashMap;
use std::io::Read;

extern crate rustc_serialize;
extern crate reqwest;

extern crate serde;
extern crate serde_json;

use self::serde_json::Map;

use util::*;


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
    let response_map: Map<String, String> = serde_json::from_str(&response).unwrap();
    match response_map.get("token") {
        Some(token) => Ok(token.clone()),
        None => Err(format!("failed to get token"))
    }

}