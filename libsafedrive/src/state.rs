use std::path::{PathBuf};

// internal imports

use ::keys::Key;
use ::models::{Token};

#[derive(Debug)]
pub struct State {
    pub storage_path: PathBuf,
    pub unique_client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_token: Option<Token>,
    pub main_key: Option<Key>,
    pub hmac_key: Option<Key>,
    pub tweak_key: Option<Key>,
}

impl State {
    pub fn new(storage_path: PathBuf, unique_client_id: String) -> State {
        State {
            storage_path: storage_path,
            unique_client_id: unique_client_id,
            username: None,
            password: None,
            api_token: None,
            main_key: None,
            hmac_key: None,
            tweak_key: None,
        }
    }

    pub fn set_keys(&mut self, main_key: Key, hmac_key: Key, tweak_key: Key) {
        self.main_key = Some(main_key);
        self.hmac_key = Some(hmac_key);
        self.tweak_key = Some(tweak_key);
    }

    pub fn get_api_token(&self) -> &Token {
        let tok = match self.api_token {
            Some(ref t) => t,
            None => panic!("attempt to use api token before setting")
        };

        tok
    }

    pub fn set_api_token(&mut self, token: Token) {
        self.api_token = Some(token);
    }


    pub fn get_unique_client_id(&self) -> &str {
        &self.unique_client_id
    }

    pub fn set_account(&mut self, username: String, password: String) {
        self.username = Some(username);
        self.password = Some(password);
    }

    pub fn get_account_username(&self) -> &str {
        let username = match self.username {
            Some(ref u) => u,
            None => panic!("attempt to use account username before setting")
        };

        username
    }

    pub fn get_account_password(&self) ->  &str {
        let password = match self.password {
            Some(ref p) => p,
            None => panic!("attempt to use account password before setting")
        };
        password

    }

    pub fn get_main_key(&self) -> &Key {
        let key = match self.main_key {
            Some(ref key) => key,
            None => panic!("Attempted to use main key before it was set")
        };
        key
    }

    pub fn get_hmac_key(&self) -> &Key {
        let key = match self.hmac_key {
            Some(ref key) => key,
            None => panic!("Attempted to use hmac key before it was set")
        };
        key
    }

    pub fn get_tweak_key(&self) -> &Key {
        let key = match self.tweak_key {
            Some(ref key) => key,
            None => panic!("Attempted to use tweak key before it was set")
        };
        key
    }
}