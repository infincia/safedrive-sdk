fn main() {
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-args=-mmacosx-version-min=10.9");
    }
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=static=sodium");
    }
}

