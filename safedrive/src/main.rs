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
use std::thread;

#[cfg(target_os = "linux")]
use std::env;

use std::path::{PathBuf};
use std::io::{Read};

extern crate clap;
use clap::{Arg, App, SubCommand};

extern crate rpassword;

extern crate pbr;
use self::pbr::MultiBar;
use self::pbr::ProgressBar;
use self::pbr::Units;

extern crate uuid;
use uuid::Uuid;

extern crate chrono;
use ::chrono::{Local, UTC, TimeZone};

extern crate number_prefix;
use ::number_prefix::{binary_prefix, Standalone, Prefixed};

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

#[cfg(target_os = "linux")]
use safedrive::core::get_openssl_directory;

#[cfg(target_os = "macos")]
use safedrive::core::unique_client_hash;

use safedrive::core::generate_unique_client_id;
use safedrive::core::get_unique_client_id;
use safedrive::core::get_app_directory;

use safedrive::core::get_current_os;


use safedrive::models::{RegisteredFolder};
use safedrive::session::SyncSession;
use safedrive::constants::Configuration;

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
    let app = App::new(NAME)
        .version(VERSION)
        .about(COPYRIGHT)
        .setting(::clap::AppSettings::SubcommandRequiredElseHelp)
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
                .required(false)
            )
        );

    let matches = app.get_matches();

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

    #[cfg(target_os = "linux")]
    {
        match get_openssl_directory() {
            Ok(ssldir) => {
                debug!("Using openssl dir: {:?}", &ssldir);

                let mut cert_dir = PathBuf::from(&ssldir);
                cert_dir.push("certs");
                let cert_dir_r: &OsStr = cert_dir.as_ref();

                let mut cert_file = PathBuf::from(&ssldir);
                cert_file.push("certs");
                cert_file.push("ca-certificates.crt");
                let cert_file_r: &OsStr = cert_file.as_ref();

                env::set_var("SSL_CERT_DIR", cert_dir_r);

                env::set_var("SSL_CERT_FILE", cert_file_r);

            },
            Err(_) => {
                error!("Could not find openssl certificate store");
                std::process::exit(1);
            }
        };
    }

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

    #[cfg(feature = "keychain")]
    let s = match ::safedrive::core::get_keychain_item(&username, ::safedrive::keychain::KeychainService::Account) {
        Ok(s) => s,
        Err(e) => {
            error!("Error getting keychain item: {}", e);
            std::process::exit(1);
        }
    };

    let uid = match get_unique_client_id(&app_directory) {
        Ok(uid) => uid,
        Err(_) => {
            #[cfg(target_os = "macos")]
            let uid = match unique_client_hash(&username) {
                Ok(hash) => hash,
                Err(e) => {
                    error!("Error generating client ID: {}", e);
                    std::process::exit(1);
                },
            };

            #[cfg(not(target_os = "macos"))]
            let uid = generate_unique_client_id();

            uid
        },
    };

    let client_version = format!("{} {}", NAME, VERSION);

    let operating_system = get_current_os();

    initialize(&client_version, operating_system, "en_US", config);

    let (token, _) = match login(&uid, &app_directory, &username, &password) {
        Ok((t, a)) => (t, a),
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





    if let Some(m) = matches.subcommand_matches("add") {
        let p = match m.value_of("path") {
            Some(p) => p,
            None => {
                error!("failed to get path from argument list");
                std::process::exit(1);
            }
        };

        let pa = PathBuf::from(&p);
        //TODO: this is not portable to windows, must be fixed before use there
        println!("Adding new sync folder {:?}",  &pa.file_name().unwrap().to_str().unwrap());

        match add_sync_folder(&token, &pa.file_name().unwrap().to_str().unwrap(), p) {
            Ok(_) => {},
            Err(e) => {
                error!("failed to add new sync folder: {}", e);
            }
        }
    } else if let Some(m) = matches.subcommand_matches("remove") {
        let id: u64 = m.value_of("id").unwrap()
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


    } else if let Some(_) = matches.subcommand_matches("syncall") {
        let folder_list = match get_sync_folders(&token) {
            Ok(fl) => fl,
            Err(e) => {
                error!("Read folders error: {}", e);
                std::process::exit(1);
            }
        };
        let encrypted_folders: Vec<RegisteredFolder> = folder_list.into_iter().filter(|f| f.encrypted).collect();

        let mut mb = MultiBar::new();
        mb.println("Syncing all folders");

        for folder in encrypted_folders {
            let mut pb = mb.create_bar(0);

            let message = format!("{}: ", folder.folderName);
            pb.message(&message);

            pb.format("╢▌▌░╟");
            pb.set_units(Units::Bytes);
            let sync_uuid = Uuid::new_v4().hyphenated().to_string();
            let local_token = token.clone();
            let local_main = keyset.main.clone();
            let local_hmac = keyset.hmac.clone();
            let local_tweak = keyset.tweak.clone();
            let local_folder_name = folder.folderName.clone();

            let _ = thread::spawn(move || {
                match sync(&local_token,
                           &sync_uuid,
                           &local_main,
                           &local_hmac,
                           &local_tweak,
                           folder.id,
                           &mut |total, _, new, _, tick, message| {
                               if message.len() > 0 {
                                   let message = format!("Stalled: ");
                                   pb.message(&message);
                               } else {
                                   let message = format!("{}: ", local_folder_name);
                                   pb.message(&message);
                               }
                               if tick {
                                   pb.tick();
                               } else {
                                   pb.total = total as u64;
                                   pb.add(new as u64);
                               }
                           }
                ) {
                    Ok(_) => {
                        let message = format!("{} finished", local_folder_name);
                        pb.finish_println(&message);
                    },
                    Err(e) => {
                        let message = format!("Sync failed: {}", e);
                        pb.finish_println(&message);
                    }
                }
            });
        }

        mb.listen();

    } else if let Some(m) = matches.subcommand_matches("sync") {
        let id: u64 = m.value_of("id").unwrap()
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
                   &mut |total, _, new, _, tick, message| {
                       if message.len() > 0 {
                           let message = format!("Stalled: ");

                           pb.message(&message);
                       }
                       if tick {
                           pb.tick();
                       } else {
                           pb.total = total as u64;
                           pb.add(new as u64);
                       }
                   }
        ) {
            Ok(_) => { pb.finish_print("Sync finished\n"); return },
            Err(e) => {
                error!("Sync error: {}", e);
                std::process::exit(1);
            }
        }
    } else if let Some(m) = matches.subcommand_matches("restore") {
        let p = m.value_of("destination").unwrap();
        let pa = PathBuf::from(&p);


        let session_list = match get_sync_sessions(&token) {
            Ok(sl) => sl,
            Err(e) => {
                error!("Read sessions error: {}", e);
                std::process::exit(1);
            }
        };


        let id: u64 = m.value_of("id").unwrap()
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

        let mut sessions: Vec<SyncSession> = session_list.into_iter().filter(|ses| ses.folder_id.unwrap() == id).collect();

        sessions.sort_by(|a, b| a.time.unwrap().cmp(&b.time.unwrap()));

        // if we got a session argument, use that one
        let mut filtered = match m.value_of("session") {
            Some(m) => {
                let sessions: Vec<SyncSession> = sessions.into_iter().filter(|ses| ses.name == m).collect();

                sessions
            },
            None => {

                sessions
            },
        };

        let ref session = match filtered.pop() {
            Some(ses) => ses,
            None => {
                error!("No session found");
                std::process::exit(1);
            },
        };

        let t = session.time.unwrap();
        let utc_time = UTC.timestamp(t as i64 / 1000, t as u32 % 1000);
        let local_time = utc_time.with_timezone(&Local);

        //TODO: this is not portable to windows, must be fixed before use there
        println!("Restoring sync folder \"{}\" ({}) to {}", &folder.folderName, &local_time, &pa.to_str().unwrap());

        let mut pb = ProgressBar::new(0);
        pb.format("╢▌▌░╟");
        pb.set_units(Units::Bytes);

        match restore(&token,
                      &session.name,
                      &keyset.main,
                      folder.id,
                      pa,
                      session.size.unwrap(),
                      &mut |total, _, new, _, tick, message| {
                          if message.len() > 0 {
                              let message = format!("Stalled: ");

                              pb.message(&message);
                          }
                          if tick {
                              pb.tick();
                          } else {
                              pb.total = total as u64;
                              pb.add(new as u64);
                          }
                      }
        ) {
            Ok(_) => { pb.finish_print("Restore finished\n"); return },
            Err(e) => {
                error!("Restore error: {}", e);
                std::process::exit(1);
            }
        }

    } else if let Some(_) = matches.subcommand_matches("list") {
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

    } else if let Some(_) = matches.subcommand_matches("sessions") {
        let mut table = Table::new();

        // Add a row
        table.add_row(row!["Folder ID", "Time", "Size", "Name"]);

        let _ = match get_sync_folders(&token) {
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
            let session_size = match binary_prefix(session.size.unwrap() as f64) {
                Standalone(bytes)   => format!("{} bytes", bytes),
                Prefixed(prefix, n) => format!("{:.2} {}B", n, prefix),
            };
            let session_folder_id = format!("{}", &session.folder_id.unwrap());

            let session_time = format!("{}", {
                let t = session.time.unwrap();
                let utc_time = UTC.timestamp(t as i64 / 1000, t as u32 % 1000);
                let local_time = utc_time.with_timezone(&Local);

                local_time
            });

            table.add_row(Row::new(vec![
                Cell::new(&session_folder_id),
                Cell::new(&session_time),
                Cell::new(&session_size),
                Cell::new(&session.name),
            ]));
        }
        table.printstd();

    }
}