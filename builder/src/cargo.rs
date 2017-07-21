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
        // config directory
        crate_dir.pop();
        // platform target directory
        crate_dir.pop();
        // target directory
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


        {
            // copy sddk
            let mut sddk_source_path: PathBuf = PathBuf::from(&target_path);
            sddk_source_path.push("sddk.dll");

            let mut sddk_destination_path: PathBuf = PathBuf::from(&out_path);
            sddk_destination_path.push("sddk.dll");

            info!("copying {} -> {}", sddk_source_path.display(), sddk_destination_path.display());
            fs::copy(&sddk_source_path, &sddk_destination_path)?;
        }

        {
            // copy import lib
            let mut import_lib_source_path: PathBuf = PathBuf::from(&target_path);
            import_lib_source_path.push("deps");
            import_lib_source_path.push("sddk.dll.lib");

            let mut import_lib_destination_path: PathBuf = PathBuf::from(&out_path);
            import_lib_destination_path.push("sddk.dll.lib");

            info!("copying {} -> {}", import_lib_source_path.display(), import_lib_destination_path.display());
            fs::copy(&import_lib_source_path, &import_lib_destination_path)?;
        }

        {
            // copy debug symbols
            let mut debug_symbols_source_path: PathBuf = PathBuf::from(&target_path);
            debug_symbols_source_path.push("deps");
            debug_symbols_source_path.push("sddk.pdb");

            let mut debug_symbols_destination_path: PathBuf = PathBuf::from(&out_path);
            debug_symbols_destination_path.push("sddk.pdb");

            info!("copying {} -> {}", debug_symbols_source_path.display(), debug_symbols_destination_path.display());
            fs::copy(&debug_symbols_source_path, &debug_symbols_destination_path)?;
        }

        {
            // copy cli
            let mut cli_source_path: PathBuf = PathBuf::from(&target_path);

            cli_source_path.push("safedrive.exe");

            let mut cli_destination_path: PathBuf = PathBuf::from(&out_path);
            // back out to top level output directory, without architecture specific component
            // since the binary is going to be renamed instead
            cli_destination_path.pop();

            let cliname = format!("safedrivecli{}.exe", self.platform.arch_width());
            cli_destination_path.push(&cliname);

            info!("copying {} -> {}", cli_source_path.display(), cli_destination_path.display());
            fs::copy(&cli_source_path, &cli_destination_path)?;
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

