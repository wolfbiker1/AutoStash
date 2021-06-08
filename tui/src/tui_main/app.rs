use crate::util::{StatefulList, TabsState};
use filewatch::FileWatch;
use std::time::Duration;
use event_handle::event_handle::EventHandle;
use store::store::Store;
const TASKS: [&str; 2] = [
    "foo.txt", "bar.dat",
];

pub struct LineDifference<'a> {
    pub name: &'a str,
    pub location: &'a str
}

pub struct App<'a> {
    pub watch_path: String,
    pub watch: FileWatch,
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub show_chart: bool,
    // pub progress: f64,
    pub tasks: StatefulList<&'a str>,
    pub servers: Vec<LineDifference<'a>>
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
            debounce_time: Duration::from_millis(debounce_time),
        })
    }
}


impl<'a> App<'a> {
    pub fn new(title: &'a str, config: &Config) -> Result<App<'a>, Box<dyn std::error::Error>> {
        let store = Store::new(config.store_path.as_str(), config.watch_path.as_str())?;
        let event_handle = EventHandle::new(store);
        let watch = FileWatch::new(config.debounce_time, event_handle)?;

        Ok (App {
            watch,
            watch_path: config.watch_path.clone(),
            title,
            should_quit: false,
            tabs: TabsState::new(vec![ "Statistic", "Info", "Overview"]),
            show_chart: true,
            // progress: 0.0,
            tasks: StatefulList::with_items(TASKS.to_vec()),

            servers: vec![
                LineDifference {
                    name: "foo",
                    location: "bar",
                },
            ],
        })
    }

    pub fn on_up(&mut self) {
        self.tasks.previous();
    }

    pub fn on_down(&mut self) {
        self.tasks.next();
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
    }
}
