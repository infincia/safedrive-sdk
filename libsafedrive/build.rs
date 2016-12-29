use std::env;

extern crate cheddar;

fn main() {
    let dist: String;
    let target = env::var("TARGET").expect("failed to get target");

    if cfg!(target_os = "windows") {
        dist = format!("..\\dist-{}\\include\\sdsync.h", target);
    }
    else {
        dist = format!("../dist-{}/include/sdsync.h", target);
    }

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-args=-mmacosx-version-min=10.9");
    }

    cheddar::Cheddar::new().expect("could not read manifest").module("c_api").expect("malformed module path").run_build(dist);
}

