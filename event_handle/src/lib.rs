pub mod event_handle {
    use std::path::PathBuf;

    use diff::LineDifference;
    use notify::DebouncedEvent;
    use std::sync::mpsc;
    use store::store::Store;

    pub struct EventHandle {
        store: Store,
        // type will be replaced soon
        tx_sorted_stack: mpsc::Sender<String>,
        tx_new_version: mpsc::Sender<String>,
    }

    impl EventHandle {
        pub fn new(
            store: Store,
            stack_transmitter: mpsc::Sender<String>,
            version_transmitter: mpsc::Sender<String>,
        ) -> EventHandle {
            EventHandle {
                store,
                tx_sorted_stack: stack_transmitter,
                tx_new_version: version_transmitter,
            }
        }

        pub fn handle(&mut self, event: DebouncedEvent) -> Result<(), Box<dyn std::error::Error>> {
            let path = self.to_path(&event)?;
            if path.is_file() {
                self.on_modification(&event)?;
                self.on_removal(&event);
            }
            Ok(())
        }

        fn on_modification(
            &mut self,
            event: &DebouncedEvent,
        ) -> Result<(), Box<dyn std::error::Error>> {
            if self.is_modification(&event) {
                return self.on_file_change(&event);
            }
            Ok(())
        }

        fn on_removal(&self, event: &DebouncedEvent) {
            if self.is_removed(&event) {
                self.on_file_remove(&event);
            }
        }

        fn to_path(&self, event: &DebouncedEvent) -> Result<PathBuf, Box<dyn std::error::Error>> {
            match event {
                DebouncedEvent::Write(p) => Ok(p.clone()),
                DebouncedEvent::Remove(p) => Ok(p.clone()),
                DebouncedEvent::Error(e, _) => Err(e.to_string().into()),
                _ => Err(format!("Event is not handled yet: {:?}", event).into()),
            }
        }

        fn on_file_change(
            &mut self,
            event: &DebouncedEvent,
        ) -> Result<(), Box<dyn std::error::Error>> {
            println!("File modified: {:?}", event);
            let path = self.to_path(event).unwrap();
            let path = path.as_path().to_str().unwrap();

            let changes = self.store.get_differences_by_path(path);
            self.tx_new_version.send(String::from("bar!"));
            let changes = diff::find_new_changes(path, &changes)?;

            self.store.store_all_differences(path, &changes)
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
