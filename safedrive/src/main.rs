#[macro_use] extern crate log;
extern crate env_logger;

use log::LogLevel;


#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::str;
use std::fs::File;

use std::path::{PathBuf};
use std::io::{Read};

extern crate clap;
use clap::{Arg, App, SubCommand};

extern crate rpassword;

extern crate pbr;
use self::pbr::ProgressBar;

extern crate uuid;
use uuid::Uuid;

extern crate safedrive;
use safedrive::core::initialize;
use safedrive::core::login;
use safedrive::core::load_keys;
use safedrive::core::create_archive;

use safedrive::util::unique_client_hash;
use safedrive::util::get_app_directory;

use safedrive::sdapi::read_folders;


#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    pub email: Option<String>,
    pub password: Option<String>,
    pub phrase: Option<String>,
}


fn main() {
    env_logger::init().unwrap();

    let matches = App::new("safedrive")
        .version("0.1")
        .about("SafeDrive")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("sets a custom config file")
            .takes_value(true)
        )
        .arg(Arg::with_name("verbose")
            .short("v")
            .multiple(true)
            .help("sets the level of verbosity")
        )
        .subcommand(SubCommand::with_name("add")
            .about("add sync folder")
            .arg(Arg::with_name("path")
                .short("p")
                .help("folder path")
            )
        )
        .subcommand(SubCommand::with_name("list")
            .about("list all registered folders")
        )
        .subcommand(SubCommand::with_name("sync")
            .about("sync all registered folder")
        )
        .get_matches();

    let app_directory = get_app_directory().expect("Error: could not determine local storage directory");
    let mut credential_file_path = PathBuf::from(&app_directory);
    credential_file_path.push("credentials.json");

    let a = app_directory.to_str().expect("Error: could not determine local storage directory");

    debug!("Using local dir: {}", &a);

    let mut credential_file = match File::open(credential_file_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Error reading account info in credentials.json: {}", e);
            std::process::exit(1);
        }
    };
    let mut cs = String::new();
    match credential_file.read_to_string(&mut cs) {
        Ok(file) => file,
        Err(e) => {
            error!("Error reading account info in credentials.json: {}", e);
            std::process::exit(1);
        }
    };

    let credentials: Credentials = match serde_json::from_str(&cs) {
        Ok(c) => c,
        Err(_) => Credentials { email: None, password: None, phrase: None }
    };

    let username = match credentials.email {
        Some(email) => email,
        None => {
            error!("No username/email found in credentials.json");
            std::process::exit(1);
        }
    };

    let password = match credentials.password {
        Some(pass) => pass,
        None => {
            error!("No password found in credentials.json");
            std::process::exit(1);
        }
    };

    let uid = match unique_client_hash(&username) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Error getting client ID: {}", e);
            std::process::exit(1);
        },
    };

    let (_, _) = initialize(a, uid);

    let (token, _, _) = match login(&username, &password) {
        Ok((t, a, ucid)) => (t, a, ucid),
        Err(e) => {
            error!("Login error: {}", e);
            std::process::exit(1);
        }
    };

    let (_, main_key, hmac_key) = match load_keys(&token, credentials.phrase, &|new_phrase| {
        // store phrase in keychain and display
        println!("NOTE: a recovery phrase has been generated for your account, please write it down somewhere safe");
        println!();
        println!("If you lose your recovery phrase you will lose access to your data!!!");
        println!("---------------------------------------------------------------------");
        println!("Recovery phrase: {}", new_phrase);
        println!("---------------------------------------------------------------------");
    }) {
        Ok((master_key, main_key, hmac_key)) => (master_key, main_key, hmac_key),
        Err(e) => {
            error!("Key error: {:?}", e);
            std::process::exit(1);
        }
    };





    if let Some(matches) = matches.subcommand_matches("add") {

    } else if let Some(matches) = matches.subcommand_matches("sync") {
        let folder_list = match read_folders(&token) {
            Ok(fl) => fl,
            Err(e) => {
                error!("Read folders error: {:?}", e);
                std::process::exit(1);
            }
        };
        for folder in folder_list {
            println!("Syncing {}", folder.folderName);
            let entry_count = 100;
            let mut pb = ProgressBar::new(entry_count);
            pb.format("╢▌▌░╟");
            let sync_uuid = Uuid::new_v4().hyphenated().to_string();
            let folder_path = PathBuf::from(&folder.folderPath);

            match create_archive(&token,
                                 &sync_uuid,
                                 &main_key,
                                 &hmac_key,
                                 folder.id as i32,
                                 folder_path,
                                 &mut |progress_percent| {
                    pb.inc();
                }) {
                Ok(_) => { pb.finish(); return },
                Err(e) => {
                    error!("Sync error: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("list") {
        let folder_list = match read_folders(&token) {
            Ok(fl) => fl,
            Err(e) => {
                error!("Read folders error: {:?}", e);
                std::process::exit(1);
            }
        };
        for folder in folder_list {
            println!("{} | ({})", folder.folderName, folder.folderPath);
        }
    }
}