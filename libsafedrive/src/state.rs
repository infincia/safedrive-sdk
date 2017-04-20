/// internal imports

use keys::Key;
use models::Token;

#[derive(Debug)]
pub struct State {
    pub unique_client_id: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_token: Option<Token>,
    pub main_key: Option<Key>,
    pub hmac_key: Option<Key>,
    pub tweak_key: Option<Key>,
}

impl State {
    pub fn new() -> State {
        State {
            unique_client_id: None,
            username: None,
            password: None,
            api_token: None,
            main_key: None,
            hmac_key: None,
            tweak_key: None,
        }
    }

    pub fn set_keys(&mut self, main_key: Option<Key>, hmac_key: Option<Key>, tweak_key: Option<Key>) {
        self.main_key = main_key;
        self.hmac_key = hmac_key;
        self.tweak_key = tweak_key;
    }

    pub fn get_api_token(&self) -> &Token {
        let tok = match self.api_token {
            Some(ref t) => t,
            None => panic!("attempt to use api token before setting"),
        };

        tok
    }

    pub fn set_api_token(&mut self, token: Option<Token>) {
        self.api_token = token;
    }

    pub fn set_unique_client_id(&mut self, unique_client_id: Option<String>) {
        self.unique_client_id = unique_client_id;
    }

    #[allow(dead_code)]
    pub fn get_unique_client_id(&self) -> &str {
        let uid = match self.unique_client_id {
            Some(ref u) => u,
            None => panic!("Attempted to use unique client id before it was set"),
        };
        uid
    }

    pub fn set_account(&mut self, username: Option<String>, password: Option<String>) {
        self.username = username;
        self.password = password;
    }

    #[allow(dead_code)]
    pub fn get_account_username(&self) -> &str {
        let username = match self.username {
            Some(ref u) => u,
            None => panic!("attempt to use account username before setting"),
        };

        username
    }

    #[allow(dead_code)]
    pub fn get_account_password(&self) ->  &str {
        let password = match self.password {
            Some(ref p) => p,
            None => panic!("attempt to use account password before setting"),
        };
        password

    }

    pub fn get_main_key(&self) -> &Key {
        let key = match self.main_key {
            Some(ref key) => key,
            None => panic!("Attempted to use main key before it was set"),
        };
        key
    }

    pub fn get_hmac_key(&self) -> &Key {
        let key = match self.hmac_key {
            Some(ref key) => key,
            None => panic!("Attempted to use hmac key before it was set"),
        };
        key
    }

    pub fn get_tweak_key(&self) -> &Key {
        let key = match self.tweak_key {
            Some(ref key) => key,
            None => panic!("Attempted to use tweak key before it was set"),
        };
        key
    }
}
