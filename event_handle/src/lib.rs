pub mod event_handle {
    use diff::LineDifference;
    use notify::DebouncedEvent;
    use std::path::PathBuf;
    use std::process;
    use std::sync::mpsc;
    use std::thread;
    use store::store::Store;
    use store::store::Version;

    pub struct EventHandle {
        store: Store,
        communication: EventHandleCommunication,
    }

    pub struct EventHandleCommunication {
        pub versions_to_ui: mpsc::Sender<Vec<Version>>,
        pub lines_to_ui: mpsc::Sender<Vec<LineDifference>>,
        pub on_undo: mpsc::Receiver<usize>,
        pub on_redo: mpsc::Receiver<usize>,
    }

    impl EventHandle {
        pub fn new(store: Store, communication: EventHandleCommunication) -> EventHandle {
            EventHandle {
                store,
                communication,
            }
        }

        pub fn send_available_data(&mut self) {
            let data = self.store.view().unwrap();
            self.communication
                .versions_to_ui
                .send(data)
                .unwrap_or_else(|err| {
                    eprintln!("Could not transmit data to TUI {:?}", err);
                    process::exit(1);
                });
        }

        pub fn on_undo(&'static mut self) {
            thread::spawn(move || loop {
                let count = self.communication.on_undo.recv().unwrap();
                &self.store.undo_by(count);
            });
        }

        pub fn on_redo(&'static mut self) {
            thread::spawn(move || loop {
                let count = self.communication.on_redo.recv().unwrap();
                &self.store.redo_by(count);
            });
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
                DebouncedEvent::NoticeWrite(p) => Ok(p.clone()),
                DebouncedEvent::Error(e, _) => Err(e.to_string().into()),
                _ => Err(format!("Event is not handled yet: {:?}", event).into()),
            }
        }

        fn on_file_change(
            &mut self,
            event: &DebouncedEvent,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let path = self.to_path(event).unwrap();
            let path = path.as_path().to_str().unwrap();

            let changes = self.store.get_changes::<LineDifference>(path);
            let changes = diff::find(path, &changes)?;
            self.communication
                .lines_to_ui
                .send(changes.clone())
                .unwrap_or_else(|err| {
                    eprintln!("Could not transmit data to TUI {:?}", err);
                    process::exit(1);
                });
            // //self.store.store_all_differences(path, &changes)
            // let foo = self.store.view();
            // let foo = foo.unwrap();

            self.store.store_changes(path, &changes)
        }

        fn on_file_remove(&self, event: &DebouncedEvent) {
            //println!("File removed: {:?}", event);
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
