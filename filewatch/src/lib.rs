extern crate notify;

use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::Path;
use std::sync::mpsc::channel;

use notify::{Watcher, RecursiveMode, watcher};

use event_handle::event_handle::EventHandle;
pub struct FileWatch {
    recv: Receiver,
    watchDog: Watcher,
}
impl FileWatch {
    pub fn new(debounce: Duration) -> Result<FileWatch, Error> {
        let (tx, rx) = channel();
        watchDog = watcher(tx, debounce)?;
        Ok(FileWatch { recv: rx, watchDog })
    }
    pub fn start_watching(&self, dir: &str) {
        watch(dir);

        loop {
            listen()
        }
    }

    fn handle(events: any) {
        for event in events {
            let handle = EventHandle::new(event);
            handle.handle();
        }
    }

    fn watch(&self, dir: &str) {
        self.watchDog.watch(dir, RecursiveMode::Recursive)?;
    }

    fn listen(&self) {
        match self.recv.recv() {
           Ok(event) => handle(event),
           Err(e) => println!("watch error: {:?}", e),
        }
    }

    fn to_path(dir: &str) -> PathBuf {
        Path::new(&dir.trim()).to_path_buf()
    }
}
