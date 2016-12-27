use std::env;

extern crate cheddar;


#[cfg(not(target_os = "windows"))]
fn main() {

    println!("cargo:rustc-link-args=-mmacosx-version-min=10.9");

    println!("cargo:rustc-link-search=native=dep-osx/lib");

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=static=sqlite3");
    println!("cargo:rustc-link-lib=static=sodium");
    println!("cargo:rustc-link-lib=static=crypto");
    println!("cargo:rustc-link-lib=static=ssl");


    cheddar::Cheddar::new().expect("could not read manifest").module("c_api").expect("malformed module path").run_build("dist-osx/include/sdsync.h");
}

#[cfg(target_os = "windows")]
fn main() {
    let bit = env::var("BIT").expect("failed to get BIT environment variable, is this appveyor?");
    let width = env::var("WIDTH").expect("failed to get WIDTH environment variable, is this appveyor?");

    println!("cargo:rustc-link-search=native=dep-win-{}-vs2015\\lib", bit);
    println!("cargo:rustc-link-search=native=C:\\Users\\appveyor\\lib{}", bit);

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=native=sqlite3");
    println!("cargo:rustc-link-lib=static=sodium");
    println!("cargo:rustc-link-lib=static=crypto");
    println!("cargo:rustc-link-lib=static=ssl");

    let dist = format!("dist-win-{}-vs2015\\include\\sdsync.h", bit);

    cheddar::Cheddar::new().expect("could not read manifest").module("c_api").expect("malformed module path").run_build(dist);
}

