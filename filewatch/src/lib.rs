extern crate notify;

use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use notify::{DebouncedEvent, ReadDirectoryChangesWatcher, RecursiveMode, Watcher, watcher};

use event_handle::event_handle::EventHandle;
pub struct FileWatch {
    recv: Receiver<DebouncedEvent>,
    watch_dog: ReadDirectoryChangesWatcher,
}
impl FileWatch {
    pub fn new(debounce_time: Duration) -> Result<FileWatch, &'static str> {
        let (tx, rx) = channel();
        let watch_dog = watcher(tx, debounce_time).unwrap();
        Ok(FileWatch { recv: rx, watch_dog })
    }
    pub fn start_watching(&mut self, dir: String) {
        self.watch(dir);

        loop {
            self.listen()
        }
    }

    fn handle(&self, event: DebouncedEvent) {
        let handle = EventHandle::new(event);
        handle.handle();
    }

    fn watch(&mut self, dir: String) {
        self.watch_dog.watch(dir, RecursiveMode::Recursive);
    }

    fn listen(&self) {
        match self.recv.recv() {
            Ok(event) => self.handle(event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
