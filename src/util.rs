use std::path::{Path, PathBuf};
use core::rustc_serialize::hex::{ToHex};


pub fn block_directory(unique_client_id: &str, hmac: &[u8]) -> PathBuf {

    let shard_identifier = hmac[0..1].to_hex().chars().next().unwrap(); //unwrap is safe here, they're always going to be 0-9 or a-f

    let mut storage_dir = Path::new("/storage/").to_owned();
    storage_dir.push(&unique_client_id);
    storage_dir.push(&Path::new("blocks"));
    storage_dir.push(shard_identifier.to_string());
    storage_dir.push(Path::new(&hmac.to_hex()));
    storage_dir
}

pub fn archive_directory(unique_client_id: &str) -> PathBuf {
    let mut storage_dir = Path::new("/storage/").to_owned();
    storage_dir.push(&unique_client_id);
    storage_dir.push(&Path::new("archives"));
    storage_dir
}

// not being used at the moment
/*fn folder_to_cfolder(folder: Folder) -> CFolder {
    let s_name = CString::new(folder.name.as_str()).unwrap();
    let s_path = CString::new(folder.path.as_str()).unwrap();
    println!("CFolder found: {:?}:{:?}", &s_name, &s_path);

    CFolder {
        id: folder.id,
        name: s_name,
        path: s_path,
    }
}*/