#[derive(Clone)]
pub struct Config {
    pub store_path: String,
    pub watch_path: String,
    pub debounce_time: Duration,
}

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
            debounce_time: Duration::from_millis(debounce_time),
        })
    }
}

use std::{env, time::Duration};
use flume;

use event_handle::event_handle::{EventHandle, EventHandleCommunication};
use filewatch::FileWatch;
use store::store::Store;

pub struct AutoStash {
    pub watch_path: String,
    pub watch: FileWatch,
}

impl AutoStash {
    pub fn new(
        config: &Config,
        communication: EventHandleCommunication,
        on_quit: flume::Receiver<()>,
    ) -> Result<AutoStash, Box<dyn std::error::Error>> {
        let store = Store::new(config.store_path.as_str(), config.watch_path.as_str())?;

        let mut event_handle = EventHandle::new(store, communication);
        event_handle.init_file_versions();
        event_handle.on_redo();
        event_handle.on_undo();
        event_handle.on_time_frame_change();
        let watch = FileWatch::new(config.debounce_time, event_handle, on_quit)?;

        Ok(AutoStash {
            watch,
            watch_path: config.watch_path.clone(),
        })
    }
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.watch.start_watching(self.watch_path.as_str())
    }
}
