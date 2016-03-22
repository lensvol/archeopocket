extern crate pocket;
extern crate rand;
extern crate ansi_term;
extern crate argparse;
extern crate rustc_serialize;

mod settings;

use pocket::Pocket;
use rand::distributions::Range;
use rand::distributions::IndependentSample;
use ansi_term::Colour::{Green, Cyan};
use argparse::{ArgumentParser, Store};

use std::io;
use settings::{Credentials, Settings, save_cfg, load_cfg};

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
    let mut cfg_fn = "archeopocket.toml".to_owned();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Dig up long-forgotten articles from your Pocket account.");
        ap.refer(&mut cfg_fn)
          .add_option(&["-c", "--cfg"],
                      Store,
                      "Load configuration from specified file.");
        ap.parse_args_or_exit();
    }

    // If there is no saved configuration present, we need at least consumer key
    // to ask Pocket for an access token.
    let mut cfg = match load_cfg(&cfg_fn) {
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

    // In case we just now acquired an access token, save it to configuration
    // file.
    if cfg.credentials.access_token.is_none() {
        let token = pocket.access_token().unwrap().to_owned();
        cfg.credentials.access_token = Some(token);
        need_save_settings = true;
    }

    if need_save_settings {
        match save_cfg(&cfg_fn, &cfg) {
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
