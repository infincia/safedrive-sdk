extern crate cheddar;

fn main() {
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-args=-mmacosx-version-min=10.9");
    }
    println!("cargo:rustc-link-lib=static=sodium");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/c_api.rs");
    println!("cargo:rerun-if-changed=../release.sh");

    cheddar::Cheddar::new().expect("could not read manifest")
        .insert_code("// THIS FILE IS AUTOGENERATED - DO NOT EDIT\n\n")
        .module("c_api").expect("malformed module path")
        .run_build("../include/sddk.h");
}
