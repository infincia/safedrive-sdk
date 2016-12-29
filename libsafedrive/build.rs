use std::env;

extern crate cheddar;

fn main() {
    let dist: &str;

    if cfg!(target_os = "macos") {
        dist = "../dist/include/sdsync.h";
    }
    else if cfg!(target_os = "windows") {
        dist = "..\\dist\\include\\sdsync.h";
    }
    else {
        dist = "../dist/include/sdsync.h";
    }

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-args=-mmacosx-version-min=10.9");
    }

    cheddar::Cheddar::new().expect("could not read manifest").module("c_api").expect("malformed module path").run_build(dist);
}

