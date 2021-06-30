use serde::Deserialize;
use std::error;
use std::time::Duration;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub store_path: String,
    pub watch_path: String,
    pub debounce_time: u64,
    pub exclude: Exclude,
}

#[derive(Clone, Deserialize)]
pub struct Exclude {
    pub paths: Vec<String>,
    pub files: Vec<String>,
}

impl Config {
    pub fn new(config_path: String) -> Result<Config, Box<dyn error::Error>> {
        let config: Config = toml::from_str(&std::fs::read_to_string(&config_path)?)?;

        Ok(config)
    }
}

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
        let store = Store::new(
            config.store_path.as_str(),
            config.watch_path.as_str(),
            config.exclude.files.clone(),
            config.exclude.paths.clone(),
        )?;

        let mut event_handle = EventHandle::new(store, communication);
        event_handle.init_file_versions();
        event_handle.on_redo();
        event_handle.on_undo();
        event_handle.on_time_frame_change();
        let watch = FileWatch::new(
            Duration::from_millis(config.debounce_time),
            event_handle,
            on_quit,
            config.exclude.files.clone(),
            config.exclude.paths.clone(),
        )?;

        Ok(AutoStash {
            watch,
            watch_path: config.watch_path.clone(),
        })
    }
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.watch.start_watching(self.watch_path.as_str())
    }
}
