use std::error::Error;
use std::time::Duration;

use filewatch::FileWatch;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut watcher = FileWatch::new(config.debounce_time).expect("Failed to initialize inotify");
    watcher.start_watching(config.path);

    Ok(())
}

pub struct Config {
    pub path: String,
    pub debounce_time: Duration
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

        let debounce_time = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a debounce time"),
        }.parse::<u64>().unwrap();

        Ok(Config { path, debounce_time: Duration::from_secs(debounce_time) })
    }
}
