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
            if is_modification(event) {
                if is_directory(event) {
                    on_dir_change(event);
                } else {
                    on_file_change(event);
                }
            } else if is_removed(event) {
                on_removed(event);
            }
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

    fn on_removed(event: any) {}

    fn on_file_change(event: any) {
        println!("file modified");
    }

    fn on_dir_change(event: any) {
        println!("Directory modified: {:?}", event.name);
    }

    fn is_modification(event: any) -> bool {
        event.mask.contains(EventMask::MODIFY)
    }

    fn is_directory(event: any) -> bool {
        event.mask.contains(EventMask::ISDIR)
    }
    fn is_removed(event: any) -> bool {
        event.mask.contains(EventMask::DELETE)
    }

    fn to_path(dir: &str) -> PathBuf {
        Path::new(&dir.trim()).to_path_buf()
    }
}
