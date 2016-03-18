extern crate pocket;
extern crate rustc_serialize;
extern crate toml;
extern crate rand;

use pocket::Pocket;
use rustc_serialize::{Decodable, Encodable};
use rand::distributions::Range;
use rand::distributions::IndependentSample;

use std::io;
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};

use std::str::from_utf8;
use toml::{decode_str, encode_str};

fn authorize(consumer_key: &str) -> Pocket {
    let mut pocket: Pocket = Pocket::new(consumer_key, None);
    let url = pocket.get_auth_url().unwrap();

    println!("Follow this link to authorize the app: {}", url);

    let mut line_input: String = "".to_owned();
    io::stdin().read_line(&mut line_input).ok();
    
    pocket.authorize().expect("Authorization failed!");
    pocket
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Credentials {
    consumer_key: String,
    access_token: Option<String>,
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Settings {
    pub credentials: Credentials,
    pub last_items: Option<usize>,
    pub max_count: Option<usize>,
}

fn load_cfg(filename: &str) -> Option<Settings> {
    let mut f = File::open(filename).unwrap();
    let mut content: Vec<u8> = vec![];
    f.read_to_end(&mut content);

    let settings = decode_str(from_utf8(&content).unwrap()).unwrap();
    Some(settings)
}

fn save_cfg(filename: &str, cfg: &Settings) {
    let content: String = encode_str(&cfg);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(filename)
        .unwrap();

    file.write_all(content.as_bytes());
}

fn acquire_pocket_instance(cfg: &Settings) -> Pocket {
    let consumer_key = &cfg.credentials.consumer_key;

    if let Some(ref token) = cfg.credentials.access_token {
        Pocket::new(
            &consumer_key,
            Some(&token),
        )
    } else {
        authorize(&consumer_key)
    }
}

fn main() {
    let mut cfg = load_cfg("archeopocket.toml").unwrap();

    let mut pocket = acquire_pocket_instance(&cfg);
    if cfg.credentials.access_token.is_none() {
        let token = pocket.access_token().unwrap().to_owned();
        cfg.credentials.access_token = Some(token);
        save_cfg("archeopocket.toml", &cfg);
    }

    let num_last_items = cfg.last_items.unwrap_or(250);
    let max_count = cfg.max_count.unwrap_or(5);
    
    let items = {
        let mut f = pocket.filter();
        f.simple();
        f.unread();
        f.count(num_last_items);
        f.get()
    };

    let result = items.unwrap();

    let between = Range::new(0, num_last_items - 1);
    let mut rng = rand::thread_rng();
    
    for _ in 0..max_count {
        let index = between.ind_sample(&mut rng);
        let item = &result[index];
        println!("Title: {}\nURL: {}\n", item.resolved_title, item.resolved_url);
    }

}
