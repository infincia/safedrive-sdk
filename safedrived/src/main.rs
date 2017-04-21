#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::{Config, TermLogger};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::path::{Path, PathBuf};

extern crate clap;
use clap::{Arg, App, SubCommand};

extern crate notify;

use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAME: &'static str = env!("CARGO_PKG_NAME");
const COPYRIGHT: &'static str = "(C) 2013-2017 SafeData Net S.R.L.";

const DEBOUNCE_DURATION: u64 = 2;

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
        match watch(&pa) {
            Ok(()) => {},
            Err(notify::Error::Generic(ref string)) => panic!("Failed to watch: {}", string),
            Err(notify::Error::Io(ioerror)) => panic!("Failed to watch: {}", ioerror),
            Err(notify::Error::PathNotFound) => panic!("Failed to watch: path not found: {}", &pa.display()),
            Err(notify::Error::WatchNotFound) => panic!("Failed to unwatch, not currently watching"),
        }
    }
}


fn watch(path: &Path) -> notify::Result<()> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(DEBOUNCE_DURATION))?;
    watcher.watch(path, RecursiveMode::Recursive)?;
    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    /// `NoticeWrite` is emitted immediately after the first write event for the path.
                    ///
                    /// If you are reading from that file, you should probably close it immediately and discard all
                    /// data you read from it.
                    DebouncedEvent::NoticeWrite(path) => {
                        println!("notice write: {}", path.display());
                    },

                    /// `NoticeRemove` is emitted immediately after a remove or rename event for the path.
                    ///
                    /// The file will continue to exist until its last file handle is closed.
                    DebouncedEvent::NoticeRemove(path) => {
                        println!("notice remove: {}", path.display());
                    },

                    /// `Create` is emitted when a file or directory has been created and no events were detected
                    /// for the path within the specified time frame.
                    ///
                    /// `Create` events have a higher priority than `Write` and `Chmod`. These events will not be
                    /// emitted if they are detected before the `Create` event has been emitted.
                    DebouncedEvent::Create(path) => {
                        println!("create: {}", path.display());
                    },

                    /// `Write` is emitted when a file has been written to and no events were detected for the path
                    /// within the specified time frame.
                    ///
                    /// `Write` events have a higher priority than `Chmod`. `Chmod` will not be emitted if it's
                    /// detected before the `Write` event has been emitted.
                    ///
                    /// Upon receiving a `Create` event for a directory, it is necessary to scan the newly created
                    /// directory for contents. The directory can contain files or directories if those contents
                    /// were created before the directory could be watched, or if the directory was moved into the
                    /// watched directory.
                    DebouncedEvent::Write(path) => {
                        println!("write: {}", path.display());
                    },

                    /// `Chmod` is emitted when attributes have been changed and no events were detected for the
                    /// path within the specified time frame.
                    DebouncedEvent::Chmod(path) => {
                        println!("chmod: {}", path.display());
                    },

                    /// `Remove` is emitted when a file or directory has been removed and no events were detected
                    /// for the path within the specified time frame.
                    DebouncedEvent::Remove(path) => {
                        println!("remove: {}", path.display());
                    },

                    /// `Rename` is emitted when a file or directory has been moved within a watched directory and
                    /// no events were detected for the new path within the specified time frame.
                    ///
                    /// The first path contains the source, the second path the destination.
                    DebouncedEvent::Rename(oldpath, newpath) => {
                        println!("rename: {} to {}", oldpath.display(), newpath.display());

                    },

                    /// `Rescan` is emitted immediately after a problem has been detected that makes it necessary
                    /// to re-scan the watched directories.
                    DebouncedEvent::Rescan => {
                        println!("rescan");
                    },

                    /// `Error` is emitted immediately after a error has been detected.
                    ///
                    ///  This event may contain a path for which the error was detected.
                    DebouncedEvent::Error(err, optpath) => {
                        print!("error: ");
                        if let Some(path) = optpath {
                            print!("({})", path.display());
                        }
                        print!("{}", err);
                        println!();
                    },
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}