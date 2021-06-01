use std::{error::Error, fs};
use std::time::Duration;

use filewatch::FileWatch;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let watcher = FileWatch::new(config.debounce).expect("Failed to initialize inotify");
    watcher.start_watching(config.path);

    Ok(())
}

pub struct Config {
    pub path: String,
    pub debounce: Duration
}

use std::env;

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        // skip binary
        args.next();

        let path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a path"),
        };

        let debounce = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a debounce"),
        }.parse::<i32>()?;

        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();

        Ok(Config { path, debounce: Duration::from_secs(debounce) })
    }
}
