use subprocess::{Exec, ExitStatus};
use platform::Platform;
use error::BuildError;
use configuration::Configuration;
use std::path::{Path, PathBuf};
use std::fs;

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
            .env("CARGO_INCREMENTAL", "1")
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

        let mut crate_dir: PathBuf = PathBuf::from(&self.build_prefix);
        // top level dep directory
        crate_dir.pop();
        // top level crate root directory
        crate_dir.pop();

        let mut out_path: PathBuf = PathBuf::from(&crate_dir);
        // solution dir
        out_path.pop();
        // configuration dir
        out_path.push(self.configuration.name());
        // platform dir
        out_path.push(self.platform.name());

        info!("creating output directory {}", out_path.display());


        if let Err(_) = fs::create_dir_all(&out_path) {
            return Err(BuildError::CommandFailed("failed to create destination dir".to_string()));
        }

        let mut target_path: PathBuf = PathBuf::from(&crate_dir);
        // target directory
        target_path.push("target");
        // triple dir
        target_path.push(self.platform.target());
        // configuration dir
        target_path.push(self.configuration.name());
        // product name
        target_path.push("sddk.dll");

        // set output path file name
        out_path.push("sddk.dll");

        info!("copying {} -> {}", target_path.display(), out_path.display());

        fs::copy(&target_path, &out_path)?;

        // switch source and target to cli app
        target_path.pop();
        target_path.push("safedrive.exe");

        // remove sddk.dll from target path so we can add the cli name
        out_path.pop();
        // back out to top level output directory, without architecture specific component
        // since the binary is going to be renamed instead
        out_path.pop();

        let cliname = format!("safedrivecli{}.exe", self.platform.arch_width());
        out_path.push(&cliname);


        info!("copying {} -> {}", target_path.display(), out_path.display());

        fs::copy(&target_path, &out_path)?;

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
            .env("CARGO_INCREMENTAL", "1")
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

