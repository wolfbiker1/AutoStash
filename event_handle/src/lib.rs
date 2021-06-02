pub mod event_handle {
    use store::store::Store;
    use diff::diff;
    use notify::DebouncedEvent;

    pub struct EventHandle {
        event: DebouncedEvent
    }

    impl EventHandle {
        pub fn new(event: DebouncedEvent) -> EventHandle {
            EventHandle{event}
        }

        pub fn handle(&self) {
            if self.is_modification() {
                if self.is_directory() {
                    self.on_dir_change();
                } else {
                    self.on_file_change();
                }
            } else if self.is_removed() {
                self.on_removed();
            }
        }

        fn on_removed(&self) {}

        fn on_file_change(&self) {
            println!("file modified");
        }
    
        fn on_dir_change(&self) {
            println!("Directory modified: {:?}", self.event);
        }
    
        fn is_modification(&self) -> bool {
            unimplemented!();
        }
    
        fn is_directory(&self) -> bool {
            unimplemented!();
        }
        fn is_removed(&self) -> bool {
            unimplemented!();
        }
    }
}
