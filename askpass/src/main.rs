extern crate sddk;
use sddk::*;

extern crate log;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {

    let mut config: Configuration = Configuration::Staging;

    match std::env::var("SAFEDRIVE_ENVIRONMENT_PRODUCTION") {
        Ok(_) => {
            config = Configuration::Production;
        },
        Err(_) => {
            // wasn't set, not an error
        }
    }

    let app_directory = match get_app_directory(&config) {
        Ok(d) => d,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let operating_system = get_current_os();

    match initialize(&VERSION, false, operating_system, "en_US", config, ::log::LogLevelFilter::Error, &app_directory) {
        Ok(()) => {},
        Err(_) => {
            std::process::exit(1);
        },
    }

    match find_credentials() {
        Ok((_, password)) => {
            print!("{}", password);
            std::process::exit(0);
        },
        Err(_) => {
            std::process::exit(1);
        },
    }
}

pub fn find_credentials() -> Result<(String, String), SDError> {

    let username = get_keychain_item("currentuser", KeychainService::CurrentUser)?;
    let password = get_keychain_item(&username, KeychainService::Account)?;

    Ok((username, password))
}