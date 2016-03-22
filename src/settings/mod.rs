extern crate toml;

use rustc_serialize::Encodable;

use std::io;
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};
use self::toml::{decode_str, encode_str};
use std::str::from_utf8;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Credentials {
    pub consumer_key: String,
    pub access_token: Option<String>,
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Settings {
    pub credentials: Credentials,
    pub last_items: Option<usize>,
    pub max_count: Option<usize>,
}

pub fn load_cfg(filename: &str) -> Option<Settings> {
    let mut f = File::open(filename).unwrap();
    let mut content: Vec<u8> = vec![];
    match f.read_to_end(&mut content) {
        Ok(_) => {
            let settings = decode_str(from_utf8(&content).unwrap()).unwrap();
            Some(settings)
        }
        Err(_) => None,
    }

}

pub fn save_cfg(filename: &str, cfg: &Settings) -> Result<usize, io::Error> {
    let content: String = encode_str(&cfg);

    let mut file = OpenOptions::new()
                       .write(true)
                       .create(true)
                       .open(filename)
                       .unwrap();

    try!(file.write_all(content.as_bytes()));

    Ok(content.len())
}
