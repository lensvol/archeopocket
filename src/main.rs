extern crate pocket;
extern crate rustc_serialize;
extern crate toml;
extern crate rand;
extern crate ansi_term;

use pocket::Pocket;
use rustc_serialize::Encodable;
use rand::distributions::Range;
use rand::distributions::IndependentSample;
use ansi_term::Colour::{Green, Cyan};

use std::io;
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};

use std::str::from_utf8;
use toml::{decode_str, encode_str};


fn read_line(prompt: &str) -> String {
    print!("{}", prompt);

    let mut line_input: String = "".to_owned();
    match io::stdin().read_line(&mut line_input) {
        Ok(_) => line_input,
        Err(_) => "".to_owned(),
    }
}

fn authorize(consumer_key: &str) -> Pocket {
    let mut pocket: Pocket = Pocket::new(consumer_key, None);
    let url = pocket.get_auth_url().unwrap();

    println!("Follow this link to authorize the app: {}", url);

    read_line("");

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
    match f.read_to_end(&mut content) {
        Ok(_) => {
            let settings = decode_str(from_utf8(&content).unwrap()).unwrap();
            Some(settings)
        }
        Err(_) => None,
    }

}

fn save_cfg(filename: &str, cfg: &Settings) -> Result<usize, io::Error> {
    let content: String = encode_str(&cfg);

    let mut file = OpenOptions::new()
                       .write(true)
                       .create(true)
                       .open(filename)
                       .unwrap();

    try!(file.write_all(content.as_bytes()));

    Ok(content.len())
}

fn acquire_pocket_instance(cfg: &Settings) -> Pocket {
    let consumer_key = &cfg.credentials.consumer_key;

    if let Some(ref token) = cfg.credentials.access_token {
        Pocket::new(&consumer_key, Some(&token))
    } else {
        authorize(&consumer_key)
    }
}

fn main() {
    let mut need_save_settings = false;

    let mut cfg = match load_cfg("archeopocket.toml") {
        Some(cfg) => cfg,
        None => {
            let consumer_key = read_line("Enter Pocket consumer key: ");
            need_save_settings = true;
            Settings {
                credentials: Credentials {
                    access_token: None,
                    consumer_key: consumer_key,
                },
                max_count: None,
                last_items: None,
            }
        }
    };

    let mut pocket = acquire_pocket_instance(&cfg);

    if cfg.credentials.access_token.is_none() {
        let token = pocket.access_token().unwrap().to_owned();
        cfg.credentials.access_token = Some(token);
        need_save_settings = true;
    }

    if need_save_settings {
        match save_cfg("archeopocket.toml", &cfg) {
            Ok(count) => println!("Wrote {} bytes to configuration file.", count),
            Err(e) => println!("Failed to save configuration file: {}", e),
        }
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
        println!("{}:\t{}\n{}:\t{}\n",
                 Cyan.paint("Title"),
                 Green.paint(&item.resolved_title[..]),
                 Cyan.paint("URL"),
                 item.resolved_url);
    }

}
