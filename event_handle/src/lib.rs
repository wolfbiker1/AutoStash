pub mod event_handle {
    use diff::LineDifference;
    use flume::{Receiver, Sender};
    use notify::DebouncedEvent;
    use store::store::FileVersions;
    use store::store::TimeFrame;
    use std::path::PathBuf;
    use std::process;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use store::store::Store;

    pub struct EventHandle {
        store: Arc<Mutex<Store>>,
        communication: Arc<EventHandleCommunication>,
    }

    pub struct EventHandleCommunication {
        pub file_versions_to_ui: Sender<Vec<FileVersions>>,
        pub on_undo: Receiver<(String, usize)>,
        pub on_redo: Receiver<(String, usize)>,
        pub on_time_frame_change: Receiver<TimeFrame>
    }

    fn transmit_file_versions(event_handle: &EventHandle) {
        let view = event_handle.store.lock().unwrap().view().unwrap();
        event_handle.communication
            .file_versions_to_ui
            .send(view)
            .unwrap_or_else(|err| {
                eprintln!("Could not transmit data to TUI {:?}", err);
                process::exit(1);
            });
    }

    impl EventHandle {
        pub fn new(store: Store, communication: EventHandleCommunication) -> EventHandle {
            EventHandle {
                store: Arc::new(Mutex::new(store)),
                communication: Arc::new(communication),
            }
        }

        pub fn init_file_versions(&self) {
            transmit_file_versions(self);
        }

        pub fn on_time_frame_change(&mut self) {
            let communication = self.communication.clone();
            let store = self.store.clone();
            thread::spawn(move || loop {
                let time_frame = communication.on_time_frame_change.recv().unwrap();
                store.lock().unwrap().change_time_frame(time_frame);
                transmit_file_versions(&EventHandle {
                    communication: communication.clone(),
                    store: store.clone()
                });
            });
        }

        pub fn on_undo(&mut self) {
            let communication = self.communication.clone();
            let store = self.store.clone();
            thread::spawn(move || loop {
                let (path, count) = communication.on_undo.recv().unwrap();
                store.lock().unwrap().undo_by(path, count).unwrap();
                transmit_file_versions(&EventHandle {
                    communication: communication.clone(),
                    store: store.clone()
                });
            });
        }

        pub fn on_redo(&mut self) {
            let communication = self.communication.clone();
            let store = self.store.clone();
            thread::spawn(move || loop {
                let (path, count) = communication.on_redo.recv().unwrap();
                store.lock().unwrap().redo_by(path, count).unwrap();
                transmit_file_versions(&EventHandle {
                    communication: communication.clone(),
                    store: store.clone()
                });
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

            let mut store = self.store.lock().unwrap();

            let changes = store.get_file_changes::<LineDifference>(path);
            let changes = diff::find(path, &changes)?;
            let stored = store.store_changes(path, &changes);
            let view = store.view()?;
            println!("{:?}", view);

            stored
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
