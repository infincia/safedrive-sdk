#[macro_use] extern crate log;
extern crate simplelog;

use ::simplelog::{Config, TermLogger};

#[macro_use] extern crate prettytable;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::str;

#[cfg(target_os = "linux")]
use std::ffi::{OsStr};

use std::fs::File;
use std::thread;

#[cfg(target_os = "linux")]
use std::env;

use std::path::{PathBuf, Path};
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

extern crate safedrive;
use safedrive::*;


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
    let app = App::new(NAME)
        .version(VERSION)
        .about(COPYRIGHT)
        .setting(::clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("bench")
            .about("benchmark chunking a file")
            .arg(Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("test file path")
                .takes_value(true)
                .required(true)
            )
            .arg(Arg::with_name("versiontwo")
                .short("2")
                .long("versiontwo")
                .help("benchmark sync version 2")
                .takes_value(false)
                .required(false)
            )
        )
        .subcommand(SubCommand::with_name("cache")
            .about("clean the local cache")
            .arg(Arg::with_name("clean")
                .short("c")
                .long("clean")
                .help("clean up old cache items if cache is > 512MiB")
                .required(true)
                .conflicts_with("remove")

            )
            .arg(Arg::with_name("remove")
                .short("r")
                .long("remove")
                .help("remove all cache items")
                .required(true)
                .conflicts_with("clean")

            )
        )
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("sets a custom config file")
            .takes_value(true)
        )
        .arg(Arg::with_name("debug")
            .short("d")
            .help("debug logging, use twice for trace logging")
            .multiple(true)
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
            .about("list sync sessions, default will list all if not given a specific folder ID")
        )
        .subcommand(SubCommand::with_name("clean")
            .about("clean sync sessions")
            .arg(Arg::with_name("id")
                .short("i")
                .long("id")
                .value_name("ID")
                .help("session ID to remove")
                .takes_value(true)
                .conflicts_with_all(&["auto", "all", "before-today", "before-date"])
                .required(true)
            )
            .arg(Arg::with_name("auto")
                .long("auto")
                .help("session ID to remove")
                .conflicts_with_all(&["id", "all", "before-today", "before-date"])
                .required(true)
            )
            .arg(Arg::with_name("all")
                .long("all")
                .help("remove all sessions")
                .conflicts_with_all(&["id", "auto", "before-today", "before-date"])
                .required(true)
            )
            .arg(Arg::with_name("before-today")
                .short("t")
                .long("before-today")
                .help("remove all sessions before today at 00:00")
                .conflicts_with_all(&["id", "auto", "all", "before-date"])
                .required(true)
            )
            .arg(Arg::with_name("before-date")
                .short("d")
                .long("before-date")
                .value_name("DATE")
                .help("delete all sessions older than this date (RFC 3339 format)\n\n")
                .conflicts_with_all(&["id", "auto", "all", "before-today"])
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("syncall")
            .about("sync all registered folders")
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

    let log_level = match matches.occurrences_of("debug") {
        0 => ::log::LogLevelFilter::Warn,
        1 => ::log::LogLevelFilter::Debug,
        2 | _ => ::log::LogLevelFilter::Trace,
    };

    let _ = TermLogger::init(log_level, Config::default()).unwrap();

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

    let channel = ::safedrive::get_channel();
    println!("Channel: {}", channel);

    match channel {
        Channel::Nightly => {
            warn!("data synced using nightly version of SafeDrive may not restore properly on stable channel");
        },
        _ => {},
    }
    println!();

    let app_directory = get_app_directory(&config).expect("Error: could not determine local storage directory");
    debug!("Using local dir: {:?}", &app_directory);

    let client_version = format!("{} {}", NAME, VERSION);

    let operating_system = get_current_os();

    match initialize(&client_version, operating_system, "en_US", config, &app_directory) {
        Ok(()) => {},
        Err(e) => {
            error!("failed to initialize libsafedrive: {}", e);
            std::process::exit(1);
        }
    }

    if let Some(m) = matches.subcommand_matches("bench") {

        let version = match m.is_present("versiontwo") {
            true => {
                println!("Benchmark: version2");
                ::safedrive::SyncVersion::Version2
            },
            false => {
                println!("Benchmark: version1");
                ::safedrive::SyncVersion::Version1
            }

        };

        let p = match m.value_of("path") {
            Some(p) => p,
            None => {
                error!("failed to get path from argument list");
                std::process::exit(1);
            }
        };

        benchmark(version, p);

    }  else if let Some(m) = matches.subcommand_matches("cache") {

        let mut result = Ok(0);

        if m.is_present("remove") {
            result = ::safedrive::clear_cache();
        } else if m.is_present("clean") {
            result = ::safedrive::clean_cache(512_000_000);
        }

        match result {
            Ok(deleted) => {
                let deleted_size = ::safedrive::pretty_bytes(deleted as f64);

                println!("cache cleaned: {}", deleted_size);
            },
            Err(e) => {
                println!("cache cleaning error: {}", e);
            }
        }


    } else if let Some(m) = matches.subcommand_matches("add") {

        let p = match m.value_of("path") {
            Some(p) => p,
            None => {
                error!("failed to get path from argument list");
                std::process::exit(1);
            }
        };

        let (token, _) = sign_in(&app_directory);

        add(token, p);

    } else if let Some(m) = matches.subcommand_matches("remove") {

        let id: u64 = m.value_of("id").unwrap()
            .trim()
            .parse()
            .expect("Expected a number");

        let (token, _) = sign_in(&app_directory);

        remove(token, id);

    } else if let Some(_) = matches.subcommand_matches("syncall") {

        let (token, keyset) = sign_in(&app_directory);

        sync_all(token, keyset);

    } else if let Some(m) = matches.subcommand_matches("sync") {

        let id: u64 = m.value_of("id").unwrap()
            .trim()
            .parse()
            .expect("Expected a number");

        let (token, keyset) = sign_in(&app_directory);

        sync_one(token, keyset, id);

    } else if let Some(m) = matches.subcommand_matches("restore") {

        let destination = m.value_of("destination").unwrap();

        let id: u64 = m.value_of("id").unwrap()
            .trim()
            .parse()
            .expect("Expected a number");

        let session_name = m.value_of("session");

        let (token, keyset) = sign_in(&app_directory);

        restore_one(token, keyset, id, destination, session_name);

    } else if let Some(_) = matches.subcommand_matches("list") {

        let (token, _) = sign_in(&app_directory);

        list_folders(token);

    } else if let Some(_) = matches.subcommand_matches("sessions") {

        let (token, _) = sign_in(&app_directory);

        list_sessions(token);

    } else if let Some(m) = matches.subcommand_matches("clean") {

        // if we got a specific session id to delete, use that
        if let Some(ids) = m.value_of("id") {
            let id: u64 = ids.trim().parse().expect("Expected a number");

            let (token, _) = sign_in(&app_directory);

            println!("Removing sync session {}", id);

            match remove_sync_session(&token, id) {
                Ok(()) => {},
                Err(e) => {
                    error!("failed to remove sync session: {}", e);
                    std::process::exit(1);
                }
            };

        } else {

            let mut schedule = SyncCleaningSchedule::Auto;

            if m.is_present("before-date") {
                if let Some(date) = m.value_of("DATE") {
                    schedule = SyncCleaningSchedule::ExactDateRFC3339 { date: date.to_string() }
                } else {
                    error!("no date given");
                    std::process::exit(1);
                }
            } else if m.is_present("before-today") {
                schedule = SyncCleaningSchedule::BeforeToday
            } else if m.is_present("all") {
                schedule = SyncCleaningSchedule::All
            } else if m.is_present("auto") {
                schedule = SyncCleaningSchedule::Auto
            };

            let (token, _) = sign_in(&app_directory);

            clean_sessions(token, schedule);
        }
    }
}



pub fn find_credentials(storage_path: &Path) -> (String, String, Option<String>) {

    let mut credential_file_path = PathBuf::from(&storage_path);
    credential_file_path.push("credentials.json");

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
    let s = match ::safedrive::get_keychain_item(&username, ::safedrive::keychain::KeychainService::Account) {
        Ok(s) => s,
        Err(e) => {
            error!("Error getting keychain item: {}", e);
            std::process::exit(1);
        }
    };

    (username, password, credentials.phrase)
}

pub fn sign_in(app_directory: &Path) -> (Token, Keyset) {

    println!("Signing in to SafeDrive...");

    let (username, password, phrase) = find_credentials(app_directory);

    let uid = match get_unique_client_id(app_directory) {
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

    let (token, _) = match login(&uid, app_directory, &username, &password) {
        Ok((t, a)) => (t, a),
        Err(e) => {
            error!("Login error: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loading keys...");

    let keyset = match load_keys(&token, phrase, &|new_phrase| {
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

    (token, keyset)
}

pub fn add(token: Token, path: &str) {

    let pa = PathBuf::from(path);

    //TODO: this is not portable to windows, must be fixed before use there
    println!("Adding new sync folder {:?}",  &pa.file_name().unwrap().to_str().unwrap());

    match add_sync_folder(&token, &pa.file_name().unwrap().to_str().unwrap(), path) {
        Ok(_) => {},
        Err(e) => {
            error!("failed to add new sync folder: {}", e);
        }
    }
}

pub fn remove(token: Token, id: u64) {

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
}

pub fn sync_all(token: Token, keyset: Keyset) {

    let folder_list = match get_sync_folders(&token) {
        Ok(fl) => fl,
        Err(e) => {
            error!("Read folders error: {}", e);
            std::process::exit(1);
        }
    };

    let encrypted_folders: Vec<RegisteredFolder> = folder_list.into_iter().filter(|f| f.encrypted).collect();

    let mut mb = MultiBar::new();

    let warnings_t: ::std::sync::Arc<::std::sync::Mutex<Vec<String>>> = ::std::sync::Arc::new(::std::sync::Mutex::new(Vec::new()));


    mb.println("Syncing all folders");

    for folder in encrypted_folders {
        let mut pb = mb.create_bar(0);

        let message = format!("{}: ", folder.folderName);
        pb.message(&message);

        pb.format("╢▌▌░╟");
        pb.set_units(Units::Bytes);
        let message = format!("{}: ", &folder.folderName);
        pb.message(&message);
        let sync_uuid = Uuid::new_v4().hyphenated().to_string();
        let local_token = token.clone();
        let local_main = keyset.main.clone();
        let local_hmac = keyset.hmac.clone();
        let local_tweak = keyset.tweak.clone();
        let local_folder_name = folder.folderName.clone();
        let warnings_t = warnings_t.clone();

        let _ = thread::spawn(move || {
            match sync(&local_token,
                       &sync_uuid,
                       &local_main,
                       &local_hmac,
                       &local_tweak,
                       folder.id,
                       &mut |total, _, new, _, tick| {
                           if tick {
                               pb.tick();
                           } else {
                               pb.total = total as u64;
                               pb.add(new as u64);
                           }
                       },
                       &mut |message| {
                           let mut warnings = warnings_t.lock().unwrap();
                           let message = format!("{}: {}", local_folder_name, message);
                           warnings.push(message);
                       }
            ) {
                Ok(_) => {
                    let message = format!("{}: finished", local_folder_name);
                    pb.finish_print(&message);
                },
                Err(e) => {
                    let message = format!("{}: sync failed: {}", local_folder_name, e);
                    pb.finish_print(&message);
                }
            }
        });
    }

    mb.listen();

    println!();

    let warnings = warnings_t.lock().unwrap();

    for warning in &**warnings {
        warn!("{}", warning);
    }
}

pub fn sync_one(token: Token, keyset: Keyset, id: u64) {

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
    let message = format!("{}: ", &folder.folderName);
    pb.message(&message);

    let sync_uuid = Uuid::new_v4().hyphenated().to_string();
    let pbt = ::std::sync::Mutex::new(pb);

    match sync(&token,
               &sync_uuid,
               &keyset.main,
               &keyset.hmac,
               &keyset.tweak,
               folder.id,
               &mut |total, _, new, _, tick| {
                   let mut pb = pbt.lock().unwrap();

                   if tick {
                       pb.tick();
                   } else {
                       pb.total = total as u64;
                       pb.add(new as u64);
                   }
               },
               &mut |message| {
                   let mut pb = pbt.lock().unwrap();

                   let message = format!("{}: {}", &folder.folderName, message);

                   pb.message_print(&message);
               }
    ) {
        Ok(_) => {
            let mut pb = pbt.lock().unwrap();

            let message = format!("{}: finished", &folder.folderName);
            pb.finish_print(&message);
        },
        Err(e) => {
            let mut pb = pbt.lock().unwrap();

            let message = format!("{}: sync failed: {}", &folder.folderName, e);
            pb.finish_print(&message);
            std::process::exit(1);
        }
    }

    println!();
}

pub fn restore_one(token: Token, keyset: Keyset, id: u64, destination: &str, session_name: Option<&str>) {

    let path = PathBuf::from(destination);

    let session_list = match get_sync_sessions(&token) {
        Ok(sl) => sl,
        Err(e) => {
            error!("Read sessions error: {}", e);
            std::process::exit(1);
        }
    };

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
    let mut filtered = match session_name {
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
    println!("Restoring sync folder \"{}\" ({}) to {}", &folder.folderName, &local_time, &path.to_str().unwrap());

    let mut pb = ProgressBar::new(0);
    pb.format("╢▌▌░╟");
    pb.set_units(Units::Bytes);
    let message = format!("{}: ", &folder.folderName);
    pb.message(&message);

    let pbt = ::std::sync::Mutex::new(pb);

    match restore(&token,
                  &session.name,
                  &keyset.main,
                  folder.id,
                  path,
                  session.size.unwrap(),
                  &mut |total, _, new, _, tick, message| {
                      let mut pb = pbt.lock().unwrap();
                      if tick {
                          pb.tick();
                      } else {
                          pb.total = total as u64;
                          pb.add(new as u64);
                      }
                  },
                  &mut |message| {
                      let mut pb = pbt.lock().unwrap();

                      let message = format!("{}: {}", &folder.folderName, message);

                      pb.message_print(&message);
                  }
    ) {
        Ok(_) => {
            let mut pb = pbt.lock().unwrap();

            let message = format!("{}: finished", &folder.folderName);
            pb.finish_print(&message);
        },
        Err(e) => {
            let mut pb = pbt.lock().unwrap();

            let message = format!("{}: restore failed: {}", &folder.folderName, e);
            pb.finish_print(&message);
            std::process::exit(1);
        }
    }

    println!();
}

pub fn list_folders(token: Token) {

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
}

pub fn list_sessions(token: Token) {

    let mut table = Table::new();

    // Add a row
    table.add_row(row!["Session ID", "Time", "Size", "Name", "Folder ID"]);

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
        let session_size = ::safedrive::pretty_bytes(session.size.unwrap() as f64);

        let session_folder_id = format!("{}", &session.folder_id.unwrap());
        let session_id = format!("{}", &session.id.unwrap());

        let session_time = format!("{}", {
            let t = session.time.unwrap();
            let utc_time = UTC.timestamp(t as i64 / 1000, t as u32 % 1000);
            let local_time = utc_time.with_timezone(&Local);

            local_time
        });

        table.add_row(Row::new(vec![
            Cell::new(&session_id),
            Cell::new(&session_time),
            Cell::new(&session_size),
            Cell::new(&session.name),
            Cell::new(&session_folder_id),

        ]));
    }
    table.printstd();
}

pub fn clean_sessions(token: Token, schedule: SyncCleaningSchedule) {

    println!("Cleaning sync sessions with schedule: {}", schedule);

    match clean_sync_sessions(&token, schedule) {
        Ok(()) => {},
        Err(e) => {
            error!("failed to clean sync sessions: {}", e);
            std::process::exit(1);
        }
    };
}


pub fn benchmark(version: ::safedrive::SyncVersion, path: &str) {

    let pa = PathBuf::from(path);

    //TODO: this is not portable to windows, must be fixed before use there
    println!("Benchmarking file {:?}",  &pa.file_name().unwrap().to_str().unwrap());

    use std::io::{BufReader, Read};

    let f = match File::open(&pa) {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to open file: {}", e);

            std::process::exit(1);
        }
    };

    let md = match f.metadata() {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to open file metadata: {}", e);

            std::process::exit(1);
        },
    };

    let stream_length = md.len();


    let reader: BufReader<File> = BufReader::new(f);
    let byte_iter = reader.bytes().map(|b| b.expect("failed to unwrap test data"));
    let tweak_key = ::safedrive::Key::new(::safedrive::KeyType::Tweak);

    let chunk_iter = ::safedrive::ChunkGenerator::new(byte_iter, &tweak_key, stream_length, version);

    let start = std::time::Instant::now();

    let mut nb_chunk = 0;

    for _ in chunk_iter {
        nb_chunk += 1;
    }
    println!("Benchmarking finished with {} chunks", nb_chunk);

    let avg = ::safedrive::pretty_bytes(stream_length as f64 / nb_chunk as f64);

    println!("Benchmarking average chunk size: {} bytes", avg);

    let speed = ::safedrive::pretty_bytes(stream_length as f64 / start.elapsed().as_secs() as f64);

    println!("Benchmarking took {} seconds", start.elapsed().as_secs());
    println!("Benchmarking throughput average: {} per second", speed);

    std::process::exit(0);
}