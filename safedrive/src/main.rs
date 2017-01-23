#[macro_use] extern crate log;
extern crate env_logger;

#[macro_use] extern crate prettytable;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

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
use self::pbr::Units;

extern crate uuid;
use uuid::Uuid;

extern crate safedrive;
use safedrive::core::initialize;
use safedrive::core::login;
use safedrive::core::load_keys;
use safedrive::core::sync;
use safedrive::core::restore;

use safedrive::core::get_sync_folders;
use safedrive::core::get_sync_folder;

use safedrive::core::add_sync_folder;
use safedrive::core::remove_sync_folder;

use safedrive::core::get_sync_sessions;
use safedrive::core::get_sync_session;


use safedrive::core::unique_client_hash;
use safedrive::core::get_app_directory;

use safedrive::models::{RegisteredFolder, Configuration};


const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAME: &'static str = env!("CARGO_PKG_NAME");
const COPYRIGHT: &'static str = "(C) 2013-2017 SafeData Net S.R.L.";


#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    pub email: Option<String>,
    pub password: Option<String>,
    pub phrase: Option<String>,
}


fn main() {
    env_logger::init().unwrap();

    let matches = App::new(NAME)
        .version(VERSION)
        .about(COPYRIGHT)
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
        .arg(Arg::with_name("production")
            .short("p")
            .conflicts_with("staging")
            .help("use the production environment")
        )
        .arg(Arg::with_name("staging")
            .short("s")
            .conflicts_with("production")
            .help("use the staging environment")
        )
        .subcommand(SubCommand::with_name("add")
            .about("add sync folder")
            .arg(Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("folder path")
                .takes_value(true)
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("remove")
            .about("remove sync folder")
            .arg(Arg::with_name("id")
                .short("i")
                .long("id")
                .value_name("ID")
                .help("folder ID")
                .takes_value(true)
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("list")
            .about("list all registered folders")
        )
        .subcommand(SubCommand::with_name("sessions")
                        .about("list all sync sessions")
        )
        .subcommand(SubCommand::with_name("syncall")
            .about("sync all registered folder")
        )
        .subcommand(SubCommand::with_name("sync")
                        .about("sync a folder")
                        .arg(Arg::with_name("id")
                            .short("i")
                            .long("id")
                            .value_name("ID")
                            .help("folder ID")
                            .takes_value(true)
                            .required(true)
                        )
        )
        .subcommand(SubCommand::with_name("restore")
            .about("restore a folder")
            .arg(Arg::with_name("id")
                .short("i")
                .long("id")
                .value_name("ID")
                .help("folder ID")
                .takes_value(true)
                .required(true)
            )
            .arg(Arg::with_name("destination")
                .short("d")
                .long("destination")
                .value_name("DESTINATION")
                .help("restore destination")
                .takes_value(true)
                .required(true)
            )
            .arg(Arg::with_name("session")
                .short("s")
                .long("session")
                .value_name("SESSION")
                .help("session to restore")
                .takes_value(true)
                .required(true)
            )
        )
        .get_matches();
    println!("{} {}", NAME, VERSION);
    println!("{}", COPYRIGHT);
    println!();
    let mut config: Configuration = Configuration::Staging;

    if matches.is_present("production") {
        println!("Environment: production");
        config = Configuration::Production;
    } else {
        println!("Environment: staging");
    }
    println!();

    let app_directory = get_app_directory(&config).expect("Error: could not determine local storage directory");
    let mut credential_file_path = PathBuf::from(&app_directory);
    credential_file_path.push("credentials.json");

    debug!("Using local dir: {:?}", &app_directory);

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
        Err(e) =>  {
            error!("Couldn't parse credentials.json: {}", e);
            std::process::exit(1);
        }
    };

    let username = match credentials.email {
        Some(email) => email,
        None => {
            error!("No email found in credentials.json");
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

    initialize(&app_directory, config);

    let (token, _, _) = match login(&uid, &username, &password) {
        Ok((t, a, ucid)) => (t, a, ucid),
        Err(e) => {
            error!("Login error: {}", e);
            std::process::exit(1);
        }
    };

    let keyset = match load_keys(&token, credentials.phrase, &|new_phrase| {
        // store phrase in keychain and display
        println!("NOTE: a recovery phrase has been generated for your account, please write it down somewhere safe");
        println!();
        println!("If you lose your recovery phrase you will lose access to your data!!!");
        println!("---------------------------------------------------------------------");
        println!("Recovery phrase: {}", new_phrase);
        println!("---------------------------------------------------------------------");
    }) {
        Ok(keyset) => keyset,
        Err(e) => {
            error!("Key error: {}", e);
            std::process::exit(1);
        }
    };





    if let Some(matches) = matches.subcommand_matches("add") {
        let p = matches.value_of("path").unwrap();
        let pa = PathBuf::from(&p);
        //TODO: this is not portable to windows, must be fixed before use there
        println!("Adding new sync folder {:?}",  &pa.file_name().unwrap().to_str().unwrap());

        match add_sync_folder(&token, &pa.file_name().unwrap().to_str().unwrap(), p) {
            Ok(_) => {},
            Err(e) => {
                error!("failed to add new sync folder: {}", e);
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("remove") {
        let id: u32 = matches.value_of("id").unwrap()
            .trim()
            .parse()
            .expect("Expected a number");
        if let Ok(folder) = get_sync_folder(&token, id) {
            println!("Removing sync folder {} ({})",  &folder.folderName, &folder.folderPath);

            match remove_sync_folder(&token, id) {
                Ok(_) => {},
                Err(e) => {
                    error!("failed to remove sync folder: {}", e);
                }
            }
        }
        else {
            println!("Could not find a registered folder for ID {}", id);
        }


    } else if let Some(matches) = matches.subcommand_matches("syncall") {
        let folder_list = match get_sync_folders(&token) {
            Ok(fl) => fl,
            Err(e) => {
                error!("Read folders error: {}", e);
                std::process::exit(1);
            }
        };
        let encrypted_folders: Vec<RegisteredFolder> = folder_list.into_iter().filter(|f| f.encrypted).collect();

        for folder in encrypted_folders {
            println!("Syncing {}", folder.folderName);

            let mut pb = ProgressBar::new(0);
            pb.format("╢▌▌░╟");
            pb.set_units(Units::Bytes);
            let sync_uuid = Uuid::new_v4().hyphenated().to_string();

            match sync(&token,
                       &sync_uuid,
                       &keyset.main,
                       &keyset.hmac,
                       &keyset.tweak,
                       folder.id,
                       &mut |total, current, new, progress_percent, tick| {
                           if tick {
                               pb.tick();
                           } else {
                               pb.total = total as u64;
                               pb.add(new as u64);
                           }
                       }
            ) {
                Ok(_) => { pb.finish(); return },
                Err(e) => {
                    error!("Sync error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("sync") {
        let id: u32 = matches.value_of("id").unwrap()
            .trim()
            .parse()
            .expect("Expected a number");

        let folder = match get_sync_folder(&token, id) {
            Ok(f) => f,
            Err(e) => {
                error!("Read folder error: {}", e);
                std::process::exit(1);
            }
        };

        //TODO: this is not portable to windows, must be fixed before use there
        println!("Syncing folder \"{}\"", &folder.folderName);

        let mut pb = ProgressBar::new(0);
        pb.format("╢▌▌░╟");
        pb.set_units(Units::Bytes);

        let sync_uuid = Uuid::new_v4().hyphenated().to_string();

        match sync(&token,
                   &sync_uuid,
                   &keyset.main,
                   &keyset.hmac,
                   &keyset.tweak,
                   folder.id,
                   &mut |total, current, new, progress_percent, tick| {
                       if tick {
                           pb.tick();
                       } else {
                           pb.total = total as u64;
                           pb.add(new as u64);
                       }
                   }
        ) {
            Ok(_) => { pb.finish(); return },
            Err(e) => {
                error!("Sync error: {}", e);
                std::process::exit(1);
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("restore") {
        let id: u32 = matches.value_of("id").unwrap()
            .trim()
            .parse()
            .expect("Expected a number");

        let session_name = matches.value_of("session").unwrap();

        let p = matches.value_of("destination").unwrap();
        let pa = PathBuf::from(&p);

        let folder = match get_sync_folder(&token, id) {
            Ok(f) => f,
            Err(e) => {
                error!("Read folder error: {}", e);
                std::process::exit(1);
            }
        };

        let session = match get_sync_session(&token, id, &session_name) {
            Ok(s) => s,
            Err(e) => {
                error!("Read session error: {}", e);
                std::process::exit(1);
            }
        };

        //TODO: this is not portable to windows, must be fixed before use there
        println!("Restoring sync folder \"{}\" ({}) to {}", &folder.folderName, &session_name, &pa.to_str().unwrap());

        let mut pb = ProgressBar::new(0);
        pb.format("╢▌▌░╟");
        pb.set_units(Units::Bytes);

        let sync_uuid = Uuid::new_v4().hyphenated().to_string();

        match restore(&token,
                      &session_name,
                      &keyset.main,
                      folder.id,
                      pa,
                      &mut |total, current, new, progress_percent, tick| {
                          if tick {
                              pb.tick();
                          } else {
                              pb.total = total as u64;
                              pb.add(new as u64);
                          }
                      }
        ) {
            Ok(_) => { pb.finish(); return },
            Err(e) => {
                error!("Restore error: {}", e);
                std::process::exit(1);
            }
        }

    } else if let Some(matches) = matches.subcommand_matches("list") {
        let mut table = Table::new();

        // Add a row
        table.add_row(row!["Name", "Path", "Encrypted", "ID"]);

        let folder_list = match get_sync_folders(&token) {
            Ok(fl) => fl,
            Err(e) => {
                error!("Read folders error: {}", e);
                std::process::exit(1);
            }
        };
        for folder in folder_list {
            table.add_row(Row::new(vec![
                Cell::new(&folder.folderName),
                Cell::new(&folder.folderPath),
                Cell::new(if folder.encrypted { "Yes" } else { "No" }),
                Cell::new(&format!("{}", &folder.id))])
            );
        }
        table.printstd();

    } else if let Some(matches) = matches.subcommand_matches("sessions") {
        let mut table = Table::new();

        // Add a row
        table.add_row(row!["Name", "Size (Bytes)", "Time", "Folder ID"]);

        let folder_list = match get_sync_folders(&token) {
            Ok(fl) => fl,
            Err(e) => {
                error!("Read folders error: {}", e);
                std::process::exit(1);
            }
        };
        let session_list = match get_sync_sessions(&token) {
            Ok(sl) => sl,
            Err(e) => {
                error!("Read sessions error: {}", e);
                std::process::exit(1);
            }
        };
        for session in session_list {
            table.add_row(Row::new(vec![
                Cell::new(&session.name),
                Cell::new(&format!("{}", &session.size.unwrap())),
                Cell::new(&format!("{}", &session.time.unwrap())),
                Cell::new(&format!("{}", &session.folder_id.unwrap()))])
            );
        }
        table.printstd();

    }
}