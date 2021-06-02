extern crate notify;

use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use notify::{watcher, DebouncedEvent, Error, ReadDirectoryChangesWatcher, RecursiveMode, Watcher};

use event_handle::event_handle::EventHandle;
pub struct FileWatch {
    event_handle: EventHandle,
    recv: Receiver<DebouncedEvent>,
    watch_dog: ReadDirectoryChangesWatcher,
}
impl FileWatch {
    pub fn new(debounce_time: Duration, event_handle: EventHandle) -> Result<FileWatch, Error> {
        let (tx, rx) = channel();
        let watch_dog = watcher(tx, debounce_time)?;
        Ok(FileWatch {
            event_handle,
            recv: rx,
            watch_dog,
        })
    }
    pub fn start_watching(&mut self, dir: &str) -> Result<(), String> {
        self.watch(dir)?;

        loop {
            match self.listen() {
                Err(e) => eprintln!("{:?}", e),
                _ => ()
            }
        }
    }

    fn handle(&self, event: DebouncedEvent) -> Result<(), String> {
        self.event_handle.handle(event)
    }

    fn watch(&mut self, dir: &str) -> Result<(), String> {
        self.watch_dog
            .watch(dir, RecursiveMode::Recursive).map_err(|err| {format!("watch error: {:?}", err)})
    }

    fn listen(&self) -> Result<(), String> {
        match self.recv.recv() {
            Ok(event) => self.handle(event),
            Err(e) => Err(format!("listen error: {:?}", e)),
        }
    }
}
