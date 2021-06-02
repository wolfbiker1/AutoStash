pub mod event_handle {
    use std::{error::Error, path::PathBuf};

    use store::store::Store;
    use diff::diff::{LineDifference, find};
    use notify::DebouncedEvent;

    pub struct EventHandle {
        store: Store
    }

    impl EventHandle {
        pub fn new(store: Store) -> EventHandle {
            EventHandle{
                store
            }
        }

        pub fn handle(&self, event: DebouncedEvent) -> Result<(), String> {
            self.on_modification(&event)?;
            self.on_removal(&event)
        }

        fn on_modification(&self, event: &DebouncedEvent) -> Result<(), String> {
            let path = self.to_path(&event)?;
            if self.is_modification(&event) {
                if self.is_directory(path) {
                    self.on_dir_change(&event);
                } else {
                    self.on_file_change(&event);
                }
            }

            Ok(())
        }

        fn on_removal(&self, event: &DebouncedEvent) -> Result<(), String> {
            let path = self.to_path(&event)?;
            if self.is_removed(&event) {
                if self.is_directory(path) {
                    self.on_dir_remove(&event);
                } else {
                    self.on_file_remove(&event);
                }
            }

            Ok(())
        }

        fn to_path(&self, event: &DebouncedEvent) -> Result<PathBuf, String> {
            match event {
                DebouncedEvent::Write(p) => Ok(p.clone()),
                DebouncedEvent::Remove(p) => Ok(p.clone()),
                DebouncedEvent::Error(e, _) => Err(e.to_string()),
                _ => Err(String::from("Event is not handled yet")),
            }
        }

        fn on_file_change(&self, event: &DebouncedEvent) {
            println!("File modified: {:?}", event);
        }

        fn on_file_remove(&self, event: &DebouncedEvent) {
            println!("File removed: {:?}", event);
        }
    
        fn on_dir_change(&self, event: &DebouncedEvent) {
            println!("Directory modified: {:?}", event);
        }

        fn on_dir_remove(&self, event: &DebouncedEvent) {
            println!("Directory removed: {:?}", event);
        }
    
        fn is_modification(&self, event: &DebouncedEvent) -> bool {
            if let DebouncedEvent::Write(_) = event {
                return true
            }
            false
        }
        fn is_directory(&self, path: PathBuf) -> bool {
            path.is_dir()
        }
        fn is_removed(&self, event: &DebouncedEvent) -> bool {
            if let DebouncedEvent::Remove(_) = event {
                return true
            }
            false
        }
    }
}
