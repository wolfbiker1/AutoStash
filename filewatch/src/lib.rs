use inotify::{EventMask, Inotify, WatchMask};
use std::path::Path;

pub fn init_inotify() -> inotify::Inotify {
    Inotify::init().expect("Failed to initialize inotify")
}

pub fn start_watching(dir_to_watch: &str, mut watchdog: inotify::Inotify) {
    let dir = Path::new(&dir_to_watch.trim()).to_path_buf();
    watchdog
        .add_watch(dir, WatchMask::MODIFY)
        .expect("Failed to add inotify watch");
    let mut buffer = [0u8; 4096];
    loop {
        let events = watchdog
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");

        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                println!("{:?}", event);
                if event.mask.contains(EventMask::ISDIR) {
                    println!("Directory modified: {:?}", event.name);
                } else {
                    // stash here
                    println!("file modified");
                }
            }
        }
    }
}
