pub mod event_handle {
    use diff::diff::FileDifferenceStore;
    pub struct EventHandle {
        event: any
    }

    impl EventHandle {
        pub fn new(event: any) -> EventHandle {
            EventHandle{event}
        }

        pub fn handle(&self) {
            if is_modification() {
                if is_directory() {
                    on_dir_change();
                } else {
                    on_file_change();
                }
            } else if is_removed() {
                on_removed();
            }
        }

        fn on_removed(&self) {}

        fn on_file_change(&self) {
            println!("file modified");
        }
    
        fn on_dir_change(&self) {
            println!("Directory modified: {:?}", self.event.name);
        }
    
        fn is_modification(&self) -> bool {
            self.event.mask.contains(EventMask::MODIFY)
        }
    
        fn is_directory(&self) -> bool {
            self.event.mask.contains(EventMask::ISDIR)
        }
        fn is_removed(&self) -> bool {
            self.event.mask.contains(EventMask::DELETE)
        }
    }
}
