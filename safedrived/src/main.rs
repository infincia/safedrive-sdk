#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::{Config, TermLogger};

#[macro_use]
extern crate prettytable;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::str;
use std::fs::File;

use std::path::{Path, PathBuf};
use std::io::Read;

extern crate clap;
use clap::{Arg, App, SubCommand};

extern crate futures;
extern crate tokio_inotify;
extern crate tokio_core;
use futures::stream::Stream;
use tokio_inotify::AsyncINotify;
use tokio_core::reactor::Core;


const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAME: &'static str = env!("CARGO_PKG_NAME");
const COPYRIGHT: &'static str = "(C) 2013-2017 SafeData Net S.R.L.";


#[derive(Debug)]
pub enum Configuration {
    Staging,
    Production,
}


fn main() {
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
        .arg(Arg::with_name("debug")
            .short("d")
            .help("debug logging")
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
        .subcommand(SubCommand::with_name("monitor")
            .about("monitor a folder for events")
            .arg(Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("folder path")
                .takes_value(true)
                .required(true)
            )
        )
        .get_matches();


    let log_level = match matches.occurrences_of("debug") {
        0 => ::log::LogLevelFilter::Warn,
        1 | _ => ::log::LogLevelFilter::Debug,
    };

    let _ = TermLogger::init(log_level, Config::default()).unwrap();

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


    if let Some(matches) = matches.subcommand_matches("monitor") {
        let p = matches.value_of("path").unwrap();
        let pa = PathBuf::from(&p);

        println!("Registering for filesystem events: {:?}",  &pa.file_name().unwrap().to_str().unwrap());
        let mut evloop = match Core::new() {
            Ok(evloop) => evloop,
            Err(e) => panic!("Failed to create event loop: {}", e),
        };

        let inot = match AsyncINotify::init(&evloop.handle()) {
            Ok(inot) => inot,
            Err(e) => panic!("Failed to get inotify handle: {}", e),
        };


        match inot.add_watch(&pa, tokio_inotify::IN_DELETE) {
            Ok(r) => {},
            Err(e) => panic!("Failed to register for folder events: {}", e),
        };

        let events = inot.for_each(|ev| {
            if ev.is_delete() {
                println!("EV: item deleted {:?}", ev.name);
            }
            Ok(())
        });

        evloop.run(events).unwrap();
    }
}