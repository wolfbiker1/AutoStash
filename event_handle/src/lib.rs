pub mod event_handle {
    use std::path::PathBuf;

    use diff::LineDifference;
    use notify::DebouncedEvent;
    use store::store::Store;

    pub struct EventHandle {
        store: Store,
    }

    impl EventHandle {
        pub fn new(store: Store) -> EventHandle {
            EventHandle { store }
        }

        pub fn handle(&mut self, event: DebouncedEvent) -> Result<(), String> {
            let path = self.to_path(&event)?;
            if path.is_file() {
                self.on_modification(&event);
                self.on_removal(&event);
            }
            Ok(())
        }

        fn on_modification(&mut self, event: &DebouncedEvent) {
            if self.is_modification(&event) {
                self.on_file_change(&event);
            }
        }

        fn on_removal(&self, event: &DebouncedEvent) {
            if self.is_removed(&event) {
                self.on_file_remove(&event);
            }
        }

        fn to_path(&self, event: &DebouncedEvent) -> Result<PathBuf, String> {
            match event {
                DebouncedEvent::Write(p) => Ok(p.clone()),
                DebouncedEvent::Remove(p) => Ok(p.clone()),
                DebouncedEvent::Error(e, _) => Err(e.to_string()),
                _ => Err(format!("Event is not handled yet: {:?}", event)),
            }
        }

        fn on_file_change(&mut self, event: &DebouncedEvent) {
            println!("File modified: {:?}", event);
            let path = self.to_path(event).unwrap();
            let path = path.to_str().unwrap();
            println!("Path is: {}", path);

            let changes = self.store.get_differences_by_path(path);
            let changes = diff::find_new_line_differences(path, &changes);

            self.store
                .store_all_differences(path, &changes)
                .unwrap_or_else(|err| println!("couldn't store new changes: {}", err));
        }

        fn on_file_remove(&self, event: &DebouncedEvent) {
            println!("File removed: {:?}", event);
        }

        fn is_modification(&self, event: &DebouncedEvent) -> bool {
            if let DebouncedEvent::Write(_) = event {
                return true;
            }
            false
        }
        fn is_removed(&self, event: &DebouncedEvent) -> bool {
            if let DebouncedEvent::Remove(_) = event {
                return true;
            }
            false
        }
    }
}
