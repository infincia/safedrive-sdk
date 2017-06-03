use std::path::Path;
use error::BuildError;
use std::fs;
use std::env;

pub fn create_dirs(build_prefix: &Path) -> Result<(), BuildError> {
    let build_dirs = vec!["lib", "include", "include/openssl"];

    for dir in build_dirs {
        let p = build_prefix.join(dir);

        if let Err(err) = fs::create_dir_all(p) {
            return Err(BuildError::Error(Box::new(err)));
        }
    }

    let p = build_prefix.join("src");

    if let Err(err) = fs::create_dir_all(p) {
        return Err(BuildError::Error(Box::new(err)));
    }

    Ok(())
}

pub fn pushd(dir: &Path) -> Result<(), BuildError> {
    info!("pushd: {}", dir.display());
    env::set_current_dir(dir)?;

    Ok(())
}

pub fn popd() -> Result<(), BuildError> {
    info!("popd");

    let mut current_dir = env::current_dir().unwrap();

    if !current_dir.pop() {
        return Err(BuildError::PopDirectoryError)
    }

    env::set_current_dir(current_dir)?;

    Ok(())
}
