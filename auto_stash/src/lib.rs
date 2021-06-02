use std::time::Duration;

use event_handle::event_handle::EventHandle;
use filewatch::FileWatch;
use store::store::Store;

pub struct AutoStash {
    watch_path: String,
    watch: FileWatch,
}

impl AutoStash {
    pub fn new(config: &Config) -> Result<AutoStash, Box<dyn std::error::Error>> {
        let store = Store::new(config.store_path.as_str());
        let event_handle = EventHandle::new(store);
        let watch = FileWatch::new(config.debounce_time, event_handle)?;

        Ok(AutoStash {
            watch,
            watch_path: config.watch_path.clone(),
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.watch.start_watching(self.watch_path.as_str())
    }
}

#[derive(Clone)]
pub struct Config {
    pub store_path: String,
    pub watch_path: String,
    pub debounce_time: Duration,
}

use std::env;

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        // skip binary
        args.next();

        let store_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a store path"),
        };

        let watch_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a watch path"),
        };

        let debounce_time = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a debounce time"),
        }
        .parse::<u64>()
        .unwrap();

        Ok(Config {
            store_path,
            watch_path,
            debounce_time: Duration::from_secs(debounce_time),
        })
    }
}
