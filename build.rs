use std::env;

extern crate cheddar;


fn main() {
    env::set_var("SODIUM_LIB_DIR", "dep-osx/lib");
    env::set_var("SODIUM_STATIC", "");

    env::set_var("SQLITE3_LIB_DIR", "dep-osx/lib");

    env::set_var("OPENSSL_STATIC", "");
    env::set_var("OPENSSL_DIR", "dep-osx/lib");

    println!("cargo:rustc-link-search=native=dep-osx/lib");

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=static=sqlite3");
    println!("cargo:rustc-link-lib=static=sodium");
    println!("cargo:rustc-link-lib=static=crypto");
    println!("cargo:rustc-link-lib=static=ssl");


    cheddar::Cheddar::new().expect("could not read manifest").module("c_api").expect("malformed module path").run_build("dist-osx/include/sdsync.h");
}
