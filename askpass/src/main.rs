fn main() {
    match std::env::var("SAFEDRIVE_PASSWORD") {
        Ok(password) => {
            print!("{}", password);            
        },
        Err(_) => {
            std::process::exit(1);
        }
    }
}

