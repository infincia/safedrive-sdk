use std::fs;
use std::path::{Path, PathBuf};
use error::BuildError;
use configuration::Configuration;
use platform::Platform;
use dir::*;
use subprocess::{ExitStatus, Exec};
use fs_extra;
use cmake::CMake;
use msbuild::MSBuild;
use walkdir::WalkDir;

use std::fs::{OpenOptions};
use std::io::{Write, Read};

use library::Library;

pub struct Dependency {
    library: Library,
    configuration: Configuration,
    platform: Platform,
    source_prefix: PathBuf,
    build_prefix: PathBuf,
}

impl Dependency {
    pub fn new(library: Library,
               configuration: Configuration,
               platform: Platform,
               source_prefix: &Path,
               build_prefix: &Path) -> Dependency {
        let d = Dependency {
            library: library,
            build_prefix: build_prefix.to_owned(),
            source_prefix: source_prefix.to_owned(),
            configuration: configuration,
            platform: platform,
        };

        d
    }
}

impl Dependency {
    pub fn build(&self) -> Result<(), BuildError> {
        info!("building dependency: {}", self.library.name());

        self.ensure_products()?;

        Ok(())
    }

    pub fn ensure_products(&self) -> Result<(), BuildError> {
        info!("ensuring products for: {}", self.library.name());

        let mut build = !self.check_version()?;

        for product_name in self.library.products() {
            let mut product_dest: PathBuf = PathBuf::from(&self.build_prefix);
            product_dest.push("lib");
            product_dest.push(product_name);

            if !product_dest.exists() {
                info!("building due to missing product: {}", product_name);
                build = true;
                break;
            }
        }

        if build {
            pushd(&self.build_prefix)?;

            let src_tarball = self.ensure_source()?;

            let src_dir = self.refresh_source(&src_tarball)?;

            pushd(&src_dir)?;

            if self.library.needs_cmake() {
                let c = CMake::new(self.library.name(), self.configuration, self.platform, &self.build_prefix);

                c.build()?;
            }

            if self.library.needs_msbuild() {
                let m = MSBuild::new(self.library.name(), self.configuration, self.platform, &self.build_prefix);

                m.build()?;
            }

            self.copy_products()?;
            self.copy_headers()?;

            popd()?;

            self.set_success()?;
        }

        Ok(())
    }

    pub fn set_success(&self) -> Result<(), BuildError> {
        info!("setting dependency built: {}:{} in {}", self.library.name(), self.library.version(), self.library.version_file());
        let mut full_path: PathBuf = PathBuf::from(&self.build_prefix);
        full_path.push(self.library.version_file());

        match OpenOptions::new().create(true).write(true).truncate(true).open(&full_path) {
            Ok(mut file) => {
                file.write_all(self.library.version().as_bytes())?;

                Ok(())
            },
            Err(err) => {
                Err(BuildError::BuildFailed(format!("could not set current library version for {} in {}: {}", self.library.name(), (&full_path).display(), err)))
            }
        }
    }

    pub fn check_version(&self) -> Result<bool, BuildError> {
        info!("checking dependency for: {}:{} in {}", self.library.name(), self.library.version(), self.library.version_file());

        let mut full_path: PathBuf = PathBuf::from(&self.build_prefix);
        full_path.push(self.library.version_file());

        match OpenOptions::new().read(true).write(false).truncate(false).open(&full_path) {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                if buf.as_str() == self.library.version() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            Err(err) => {
                Ok(false)
            }
        }
    }

    pub fn ensure_source(&self) -> Result<PathBuf, BuildError> {
        info!("ensuring source for: {}", self.library.name());

        let mut src_tarball: PathBuf = PathBuf::from(&self.source_prefix);
        src_tarball.push(format!("{}-{}.tar.gz", self.library.name(), self.library.version()));

        if src_tarball.exists() {
            info!("found source for: {}", self.library.name());
            return Ok(src_tarball);
        }
        let exit = Exec::cmd("curl")
            .arg("-L")
            .arg(self.library.url())
            .arg("-o")
            .arg(src_tarball.as_os_str())
            .join()?;
        match exit {
            ExitStatus::Exited(0) => {

            },
            _ => {
                return Err(BuildError::SourceMissing(format!("{}", self.library.name())));
            }
        }
        Ok(src_tarball)
    }

    pub fn refresh_source(&self, src_tarball: &Path) -> Result<PathBuf, BuildError> {
        info!("removing old source for: {}", self.library.name());

        let source_dir_name = format!("{}-{}", self.library.name(), self.library.version());

        let mut source_dir = PathBuf::from(&self.build_prefix);
        source_dir.push(&source_dir_name);

        info!("removing directory: {}", &source_dir.display());

        match fs::remove_dir_all(&source_dir) {
            _ => {}
        }

        let zc = format!("7z x -y {}", src_tarball.display());
        info!("running: {}", zc);

        let exit = Exec::shell(&zc).join()?;
        match exit {
            ExitStatus::Exited(0) => {

            },
            _ => {
                return Err(BuildError::CommandFailed("failed to unpack source".to_string()));
            }
        }

        let zc = format!("7z x -y {}-{}.tar", self.library.name(), self.library.version());
        info!("running: {}", zc);

        let exit = Exec::shell(&zc).join()?;

        match exit {
            ExitStatus::Exited(0) => {

            },
            _ => {
                return Err(BuildError::CommandFailed("failed to unpack source".to_string()));
            }
        }

        Ok(source_dir)
    }


    pub fn copy_headers(&self) -> Result<(), BuildError> {
        info!("copying headers for: {}", self.library.name());

        let options = fs_extra::dir::CopyOptions::new();
        let handle = |process_info: fs_extra::TransitProcess| {
            fs_extra::dir::TransitProcessResult::OverwriteAll
        };

        let header_names = self.library.headers();
        let headers: Vec<PathBuf> = header_names.into_iter().map(|header_name| {
            let mut p: PathBuf = PathBuf::from(&self.build_prefix);
            p.push(format!("{}-{}", self.library.name(), self.library.version()));
            p.push("include");
            p.push(header_name);
            info!("copying {}", (&p).display());

            p

        }).collect();


        let mut target = PathBuf::from(&self.build_prefix);
        target.push("include");

        fs_extra::copy_items_with_progress(&headers, &target, &options, handle)?;

        Ok(())
    }

    fn copy_products(&self) -> Result<(), BuildError> {

        info!("copying products for: {}", self.library.name());

        let products: Vec<&str> = self.library.products();

        for product in products {
            info!("searching for: {}", product);
            let mut found = false;

            let mut product_location: PathBuf = PathBuf::from(&self.build_prefix);
            product_location.push(format!("{}-{}", self.library.name(), self.library.version()));

            if ! product_location.exists() {

                return Err(BuildError::SourceMissing(product.to_string()));
            }

            let mut product_dest: PathBuf = PathBuf::from(&self.build_prefix);
            product_dest.push("lib");
            product_dest.push(product);

            for item in WalkDir::new(&product_location).into_iter().filter_map(|e| e.ok()) {
                let item_path: &Path = item.path();

                if item_path.file_name().unwrap().to_str().unwrap() == product {
                    info!("copying {} -> {}", item_path.display(), (&product_dest).display());

                    fs::copy(item_path, &product_dest)?;

                    if product.starts_with("lib") {
                        product_dest.pop();
                        let (_, libname) = product.split_at(3);

                        product_dest.push(format!("{}", libname));
                        info!("copying {} -> {}", item_path.display(), (&product_dest).display());

                        fs::copy(item_path, &product_dest)?;

                    }

                    found = true;
                }
            }

            if ! found {
                warn!("copying {} failed", product);

                return Err(BuildError::BuiltProductMissing(product.to_string()));
            }
        }

        Ok(())
    }
}
