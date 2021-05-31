use std::{error::Error, fs};

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.path)?;

    let notifier = filewatch::init_inotify();
    filewatch::start_watching("./", notifier);

    Ok(())
}

pub struct Config {
    pub path: String,
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

        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();

        Ok(Config { path })
    }
}
