extern crate notify;

use flume;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use notify::{watcher, DebouncedEvent, Error, RecommendedWatcher, RecursiveMode, Watcher};

use event_handle::event_handle::EventHandle;

pub struct FileWatch {
    event_handle: EventHandle,
    on_event: Receiver<DebouncedEvent>,
    on_quit: flume::Receiver<()>,
    watch_dog: RecommendedWatcher,
}
impl FileWatch {
    pub fn new(
        debounce_time: Duration,
        event_handle: EventHandle,
        on_quit: flume::Receiver<()>,
    ) -> Result<FileWatch, Error> {
        let (tx, on_event) = channel();
        let watch_dog = watcher(tx, debounce_time)?;
        Ok(FileWatch {
            event_handle,
            on_event,
            watch_dog,
            on_quit,
        })
    }
    pub fn start_watching(&mut self, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.watch(dir)?;

        loop {
            self.listen()?;

            if let Ok(_) = self.on_quit.try_recv() {
                break;
            }
        }

        Ok(())
    }

    fn watch(&mut self, dir: &str) -> Result<(), notify::Error> {
        self.watch_dog.watch(dir, RecursiveMode::Recursive)
    }

    fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(event) = self.on_event.try_recv() {
            return self.handle(event);
        }
        Ok(())
    }

    fn handle(&mut self, event: DebouncedEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.event_handle.handle(event)
    }
}
