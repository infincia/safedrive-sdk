use std::str;
use std::{thread, time};

extern crate clap;
use clap::{Arg, App, SubCommand};

extern crate rpassword;

extern crate pbr;
use self::pbr::ProgressBar;

extern crate safedrive;
use safedrive::core::initialize;
use safedrive::core::login;

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

    let folder_list = match read_folders(&token) {
        Ok(fl) => fl,
        Err(e) => {
            println!("Read folders error: {:?}", e);
            return
        }
    };
    for folder in folder_list {
        println!("Registered folder: {:?}", folder);
    }

    if let Some(matches) = matches.subcommand_matches("add") {

    } else if let Some(matches) = matches.subcommand_matches("sync") {

        let entry_count = 5000;
        let mut pb = ProgressBar::new(entry_count);
        pb.format("╢▌▌░╟");

        let mill = time::Duration::from_millis(100);

        for entry in 1..entry_count {
            pb.inc();
            thread::sleep(mill);
        }

        pb.finish();
    }
}