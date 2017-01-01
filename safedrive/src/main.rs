use std::str;
use std::{thread, time};

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


fn main() {
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
    let a = app_directory.to_str().expect("Error: could not determine local storage directory");

    println!("Using local dir: {}", &a);


    // prompt for account
    let username = rpassword::prompt_password_stdout("Username: ").unwrap();
    let password = rpassword::prompt_password_stdout("Password: ").unwrap();

    let uid = match unique_client_hash(&username) {
        Ok(hash) => hash,
        Err(e) => return,
    };

    let (db_path, storage_path, unique_client_path, unique_client_id) = initialize(a, uid);

    let (token, account_status, _) = match login(&username, &password) {
        Ok((t, a, ucid)) => (t, a, ucid),
        Err(e) => {
            println!("Login error: {}", e);
            return
        }
    };

    let phrase = "safedrive".to_owned();
    let ssh_username = account_status.userName;
    let ssh_host = account_status.host;
    let ssh_port = account_status.port;

    let (_, main_key, hmac_key) = match load_keys(&token, Some(phrase), &|new_phrase| {
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
            println!("Key error: {}", e);
            return 1
        }
    };





    if let Some(matches) = matches.subcommand_matches("add") {

    } else if let Some(matches) = matches.subcommand_matches("sync") {
        let folder_list = match read_folders(&token) {
            Ok(fl) => fl,
            Err(e) => {
                println!("Read folders error: {:?}", e);
                return
            }
        };
        for folder in folder_list {
            println!("syncing {}", folder.folderName);
            let entry_count = 100;
            let mut pb = ProgressBar::new(entry_count);
            pb.format("╢▌▌░╟");
            let sync_uuid = Uuid::new_v4().hyphenated().to_string();

            match create_archive(&sync_uuid,
                                 &main_key,
                                 &hmac_key,
                                 &ssh_username,
                                 &password,
                                 &ssh_host,
                                 ssh_port,
                                 &unique_client_id,
                                 db_path,
                                 folder.id as i32, &mut |progress_percent| {
                    pb.inc();
                }) {
                Ok(_) => { pb.finish(); return },
                Err(e) => {
                    println!("Sync error: {:?}", e);
                    return
                }
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("list") {
        let folder_list = match read_folders(&token) {
            Ok(fl) => fl,
            Err(e) => {
                println!("Read folders error: {:?}", e);
                return 1
            }
        };
        for folder in folder_list {
            println!("Name: {} ({})", folder.folderName, folder.folderPath);
        }
    }
}