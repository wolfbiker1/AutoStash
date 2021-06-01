use event_handle::event_handle::EventHandle;
use inotify::{EventMask, Inotify, WatchMask};
use std::path::Path;
pub struct FileWatch {
    watchDog: inotify::Inotify,
}
impl FileWatch {
    pub fn new() -> Result<FileWatch, Error> {
        watchDog = Inotify::init()?;
        Ok(FileWatch { watchDog })
    }
    pub fn start_watching(&self, dir: &str) {
        watch(dir);

        let mut buffer = [0u8; 4096];
        loop {
            let events = listen(buffer);
            handle(events);
        }
    }

    fn handle(events: any) {
        for event in events {
            let handle = EventHandle::new(event);
            handle.handle();
        }
    }

    fn watch(&self, dir: &str) {
        self.watchdog
            .add_watch(to_path(dir), WatchMask::MODIFY)
            .expect("Failed to add inotify watch");
    }

    fn listen(&self, &mut buffer: [u8; 4096]) {
        self.watchdog
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");
    }

    fn to_path(dir: &str) -> PathBuf {
        Path::new(&dir.trim()).to_path_buf()
    }
}
