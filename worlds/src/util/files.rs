use std::{fs::File, io::Read};

pub fn load_file_bytes(path: &str) -> Vec<u8> {
    let mut f = File::open(path).expect("no file found");
    let metadata = std::fs::metadata(path).expect("unable to read metadata");

    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}