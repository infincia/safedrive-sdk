#![allow(non_camel_case_types)]

use std::path::{PathBuf, Path};
use std::env;
use std::fs::OpenOptions;
use std::io::Read;

#[macro_use]
extern crate log;
extern crate simplelog;
extern crate clap;
extern crate subprocess;
extern crate fs_extra;
extern crate walkdir;

use simplelog::{Config, TermLogger};
use clap::{Arg, App, SubCommand};
use subprocess::{Exec, ExitStatus};


mod error;
mod platform;
mod runtime;
mod dependency;
mod configuration;
mod toolset;
mod cargo;
mod msbuild;
mod cmake;
mod dir;
mod library;

use error::BuildError;
use platform::Platform;
use dependency::*;
use configuration::Configuration;
use toolset::Toolset;
use cargo::Cargo;
use dir::*;
use library::Library;


const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const COPYRIGHT: &str = "(C) 2013-2017 SafeData Net S.R.L.";

fn main() {
    let matches = App::new(NAME)
        .version(VERSION)
        .about(COPYRIGHT)
        .arg(Arg::with_name("verbose")
            .short("v")
            .help("verbose logging")
        )
        .arg(Arg::with_name("platform")
            .short("p")
            .long("platform")
            .value_name("PLATFORM")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("configuration")
            .short("c")
            .long("configuration")
            .value_name("CONFIGURATION")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("toolset")
            .short("t")
            .long("toolset")
            .value_name("TOOLSET")
            .takes_value(true)
            .required(true)
        )
        .subcommand(SubCommand::with_name("build")
            .about("build sddk")
        )
        .subcommand(SubCommand::with_name("test")
            .about("test sddk")
        )
        .get_matches();


    let log_level = match matches.occurrences_of("verbose") {
        0 => ::log::LogLevelFilter::Info,
        1 | _ => ::log::LogLevelFilter::Debug,
    };

    let _ = TermLogger::init(log_level, Config::default()).unwrap();

    info!("{} {}", NAME, VERSION);
    info!("{}", COPYRIGHT);
    info!("");

    let configuration_s = matches.value_of("configuration").unwrap().to_string();

    let configuration = Configuration::from(configuration_s);
    let current_dir = env::current_dir().unwrap();

    let platform_s = matches.value_of("platform").unwrap().to_string();
    let platform = Platform::from(platform_s);

    let toolset_s = matches.value_of("toolset").unwrap().to_string();
    let toolset = Toolset::from(toolset_s);


    let mut build_prefix: PathBuf = PathBuf::from(current_dir.clone());
    build_prefix.push(platform.name());
    build_prefix.push(configuration.name());

    let mut source_prefix: PathBuf = PathBuf::from(current_dir.clone());
    source_prefix.push("src");

    info!("Building in {} for {} {} {}", current_dir.display(), platform.name(), toolset.name(), configuration.name());


    match add_rust_target(platform) {
        Ok(()) => {

        },
        Err(err) => {
            warn!("{}", err);
        }
    }

    match set_rust_version(&current_dir) {
        Ok(()) => {

        },
        Err(err) => {
            error!("{}", err);
            std::process::exit(1);
        }
    }

    match create_dirs(&build_prefix, &source_prefix) {
        Ok(()) => {

        },
        Err(err) => {
            error!("{}", err);
            std::process::exit(1);
        }
    }

    let libsodium = Dependency::new(Library::Libsodium,
                                    configuration,
                                    platform,
                                    &source_prefix,
                                    &build_prefix,
    );

    match libsodium.build() {
        Ok(()) => {},
        Err(err) => {
            error!("libsodium build failed: {}", err);
            std::process::exit(1);
        }
    }

    let libressl = Dependency::new(Library::Libressl,
                                   configuration,
                                   platform,
                                   &source_prefix,
                                   &build_prefix,

    );

    match libressl.build() {
        Ok(()) => {},
        Err(err) => {
            error!("libressl build failed: {}", err);
            std::process::exit(1);
        }
    }

    let libssh2 = Dependency::new(Library::Libssh2,
                                  configuration,
                                  platform,
                                  &source_prefix,
                                  &build_prefix,
    );

    match libssh2.build() {
        Ok(()) => {},
        Err(err) => {
            error!("libssh2 build failed: {}", err);
            std::process::exit(1);
        }
    }


    if let Some(m) = matches.subcommand_matches("build") {

        let sddk = Cargo::new(configuration,
                              platform,
                              &build_prefix,
        );

        match sddk.build() {
            Ok(()) => {},
            Err(err) => {
                error!("sddk build failed: {}", err);
                std::process::exit(1);
            }
        }
    }

    if let Some(m) = matches.subcommand_matches("test") {

        let sddk = Cargo::new(configuration,
                              platform,
                              &build_prefix,
        );

        match sddk.test() {
            Ok(()) => {},
            Err(err) => {
                error!("sddk test failed: {}", err);
                std::process::exit(1);
            }
        }
    }
}

fn set_rust_version(current_dir: &Path) -> Result<(), BuildError> {

    let mut full_path: PathBuf = PathBuf::from(current_dir);
    full_path.push("rustver.conf");

    let mut file = OpenOptions::new().read(true).write(false).truncate(false).open(&full_path)?;
    let mut ver = String::new();
    file.read_to_string(&mut ver)?;
    info!("setting rust version to: {}", ver);

    let exit = Exec::shell(format!("rustup override set {}", ver)).join()?;
    match exit {
        ExitStatus::Exited(0) => {
            Ok(())
        },
        _ => {
            Err(BuildError::CommandFailed("rust override could not be set".to_string()))
        }
    }
}

fn add_rust_target(platform: Platform) -> Result<(), BuildError> {
    info!("adding rust target: {}", platform.target());

    let _ = Exec::shell(format!("rustup target add {} > NUL 2>&1", platform.target())).capture()?;

    Ok(())
}







