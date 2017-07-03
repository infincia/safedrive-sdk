use subprocess::{Exec, ExitStatus};
use platform::Platform;
use error::BuildError;
use configuration::Configuration;
use std::path::{Path, PathBuf};

pub struct Cargo {
    configuration: Configuration,
    platform: Platform,
    build_prefix: PathBuf,
}

impl Cargo {
    pub fn new(configuration: Configuration,
               platform: Platform,
               build_prefix: &Path) -> Cargo {
        Cargo {
            build_prefix: build_prefix.to_owned(),
            configuration: configuration,
            platform: platform,
        }
    }
}

impl Cargo {
    pub fn build(&self) -> Result<(), BuildError> {
        info!("building sddk");

        let mut lib_dir = PathBuf::from(&self.build_prefix);
        lib_dir.push("lib");

        let mut exec = Exec::cmd("cargo")
            .env("OPENSSL_DIR", (&self.build_prefix).as_os_str())
            .env("BUILD_PREFIX", (&self.build_prefix).as_os_str())
            .env("SODIUM_LIB_DIR", (&lib_dir).as_os_str())
            .env("SODIUM_STATIC", "1")
            .env("RUST_BACKTRACE", "1")

            .arg("build")
            .arg("--verbose")
            .arg("--package")
            .arg("safedrive")
            .arg("--target")
            .arg(self.platform.target());

        match self.configuration {
            Configuration::Release => {
                exec = exec.arg("--release");
            },
            _ => {}
        }
        let exit = exec.join()?;

        match exit {
            ExitStatus::Exited(0) => {

            },
            _ => {
                return Err(BuildError::CommandFailed("failed to build sddk".to_string()));
            }
        }

        Ok(())
    }

    pub fn test(&self) -> Result<(), BuildError> {
        info!("testing sddk");

        let mut lib_dir = PathBuf::from(&self.build_prefix);
        lib_dir.push("lib");

        let mut exec = Exec::cmd("cargo")
            .env("OPENSSL_DIR", (&self.build_prefix).as_os_str())
            .env("BUILD_PREFIX", (&self.build_prefix).as_os_str())
            .env("SODIUM_LIB_DIR", (&lib_dir).as_os_str())
            .env("SODIUM_STATIC", "1")
            .env("RUST_BACKTRACE", "1")

            .arg("test")
            .arg("--verbose")
            .arg("--package")
            .arg("sddk")
            .arg("--target")
            .arg(self.platform.target());

        match self.configuration {
            Configuration::Release => {
                exec = exec.arg("--release");
            },
            _ => {}
        }
        let exit = exec.join()?;

        match exit {
            ExitStatus::Exited(0) => {

            },
            _ => {
                return Err(BuildError::CommandFailed("failed to test sddk".to_string()));
            }
        }

        Ok(())
    }
}

