/// internal imports

use keys::Key;
use models::Token;

#[derive(Debug)]
pub struct State {
    pub unique_client_id: Option<String>,
    pub unique_client_name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub ssh_username: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_token: Option<Token>,
    pub master_key: Option<Key>,
    pub main_key: Option<Key>,
    pub hmac_key: Option<Key>,
    pub tweak_key: Option<Key>,
}

impl State {
    pub fn new() -> State {
        State {
            unique_client_id: None,
            unique_client_name: None,
            host: None,
            port: None,
            ssh_username: None,
            username: None,
            password: None,
            api_token: None,
            master_key: None,
            main_key: None,
            hmac_key: None,
            tweak_key: None,
        }
    }

    pub fn set_keys(&mut self, master_key: Option<Key>, main_key: Option<Key>, hmac_key: Option<Key>, tweak_key: Option<Key>) {
        self.master_key = master_key;
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

    pub fn set_unique_client_name(&mut self, unique_client_name: Option<String>) {
        self.unique_client_name = unique_client_name;
    }

    #[allow(dead_code)]
    pub fn get_unique_client_name(&self) -> &str {
        let uns = match self.unique_client_name {
            Some(ref u) => u,
            None => panic!("Attempted to use unique client name before it was set"),
        };
        uns
    }

    pub fn set_host(&mut self, host: Option<String>) {
        self.host = host;
    }

    #[allow(dead_code)]
    pub fn get_host(&self) -> &str {
        let host = match self.host {
            Some(ref host) => host,
            None => panic!("Attempted to use host before it was set"),
        };
        host
    }

    pub fn set_port(&mut self, port: Option<u16>) {
        self.port = port;
    }

    #[allow(dead_code)]
    pub fn get_port(&self) -> u16 {
        let port = match self.port {
            Some(port) => port,
            None => panic!("Attempted to use port before it was set"),
        };
        port
    }

    pub fn set_ssh_username(&mut self, ssh_username: Option<String>) {
        self.ssh_username = ssh_username;
    }

    #[allow(dead_code)]
    pub fn get_ssh_username(&self) -> &str {
        let ssh_username = match self.ssh_username {
            Some(ref ssh_username) => ssh_username,
            None => panic!("Attempted to use ssh username before it was set"),
        };
        ssh_username
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

    #[allow(dead_code)]
    pub fn get_master_key(&self) -> &Key {
        let key = match self.master_key {
            Some(ref key) => key,
            None => panic!("Attempted to use master key before it was set"),
        };
        key
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
