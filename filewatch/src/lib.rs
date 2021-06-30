extern crate notify;

use flume;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use notify::{watcher, DebouncedEvent, Error, RecommendedWatcher, RecursiveMode, Watcher};

use event_handle::event_handle::EventHandle;

pub struct FileWatch {
    event_handle: EventHandle,
    on_event: Receiver<DebouncedEvent>,
    on_quit: flume::Receiver<()>,
    watch_dog: RecommendedWatcher,
    excluded_files: Vec<String>,
    excluded_paths: Vec<String>,
}
impl FileWatch {
    pub fn new(
        debounce_time: Duration,
        event_handle: EventHandle,
        on_quit: flume::Receiver<()>,
        excluded_files: Vec<String>,
        excluded_paths: Vec<String>,
    ) -> Result<FileWatch, Error> {
        let (tx, on_event) = channel();
        let watch_dog = watcher(tx, debounce_time)?;
        Ok(FileWatch {
            event_handle,
            on_event,
            watch_dog,
            on_quit,
            excluded_files,
            excluded_paths,
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

    // TODO exclude paths
    fn handle(&mut self, event: DebouncedEvent) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.to_path(&event)?;
        if path.is_file() && !self.excluded_files.contains(&path.to_str().unwrap().to_string()) {
            return self.event_handle.handle(event);
        }

        Ok(())
    }

    fn to_path(&self, event: &DebouncedEvent) -> Result<PathBuf, Box<dyn std::error::Error>> {
        match event {
            DebouncedEvent::Write(p) => Ok(p.clone()),
            DebouncedEvent::Remove(p) => Ok(p.clone()),
            DebouncedEvent::NoticeWrite(p) => Ok(p.clone()),
            DebouncedEvent::Error(e, _) => Err(e.to_string().into()),
            _ => Err(format!("Event is not handled yet: {:?}", event).into()),
        }
    }
}
