use std::env;

extern crate cheddar;

fn main() {
    let dist: String;
    let target = env::var("TARGET").expect("failed to get target");

    if cfg!(target_os = "windows") {
        let toolset = env::var("TOOLSET").expect("failed to get toolset");
        let linktype = env::var("LINKTYPE").expect("failed to get link type");
        dist = format!("..\\dist-{}-{}-{}\\include\\sddk.h", target, toolset, linktype);
    }
    else {
        dist = format!("../dist-{}/include/sddk.h", target);
    }

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-args=-mmacosx-version-min=10.9");
    }

    cheddar::Cheddar::new().expect("could not read manifest").module("c_api").expect("malformed module path").run_build(dist);
}

