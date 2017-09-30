fn main() {
    match std::env::var("SAFEDRIVE_PASSWORD") {
        Ok(password) => {
            print!("{}", password);            
        },
        Err(err) => {
            std::process::exit(1);
        }
    }
}

