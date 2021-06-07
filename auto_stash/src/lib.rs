use std::time::Duration;

use event_handle::event_handle::EventHandle;
use filewatch::FileWatch;
use std::process;

use store::store::Store;
use tui;
use std::sync::mpsc;

pub struct AutoStash {
    watch_path: String,
    watch: FileWatch,
    // channels: Channels
    // rx_sorted_stack:&'static mpsc::Receiver<String>,
    // rx_new_version: &'static mpsc::Receiver<String>,
}

// struct Channels {
//     tx_sorted_stack: mpsc::Sender<String>,
//     tx_new_version: mpsc::Sender<String>,
//     rx_sorted_stack: &'static mpsc::Receiver<String>,
//     rx_new_version: &'static mpsc::Receiver<String>,
// }


// impl Channels {
//     pub fn new() -> Result<Channels, Box<dyn std::error::Error>> {
//         let (tx, rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
//         let (tx1, rx1): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
//         Ok(Channels {
//             tx_sorted_stack: tx,
//             tx_new_version: tx1,
//             rx_sorted_stack: &rx,
//             rx_new_version: &rx1,
//         })
//     }
// }

impl AutoStash {
    pub fn new(config: &Config, t1: mpsc::Sender<String>, t2: mpsc::Sender<String>) -> Result<AutoStash, Box<dyn std::error::Error>> {
        let store = Store::new(config.store_path.as_str(), config.watch_path.as_str())?;
        
        // let ch = Channels::new().unwrap();

        let event_handle = EventHandle::new(store, t1, t2);
        let watch = FileWatch::new(config.debounce_time, event_handle)?;

        Ok(AutoStash {
            watch,
            watch_path: config.watch_path.clone(),
            // rx_new_version: ch.rx_new_version,
            // rx_sorted_stack: ch.rx_sorted_stack
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        // let t = thread::spawn(move || {
        //     tui::run_tui(self.rx_sorted_stack, self.rx_new_version).unwrap_or_else(|err| {
        //         eprintln!("Could not run tui! {:?}", err);
        //         process::exit(1);
        //     });
        // });

        // t.join().unwrap_or_else(|err| {
        //     eprintln!("Could not join thread {:?}", err);
        //     process::exit(1);
        // });
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
            debounce_time: Duration::from_millis(debounce_time),
        })
    }
}
