fn main() {
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-args=-mmacosx-version-min=10.9");
    }
}

