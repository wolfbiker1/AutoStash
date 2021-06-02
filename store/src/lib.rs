pub mod store {
    use diff::diff::LineDifference;
    use std::collections::VecDeque;
    pub struct Store {
        line_differences: VecDeque<LineDifference>,
        store_path: String,
    }

    impl Store {
        pub fn new(store_path: String) -> Store {
            Store {
                line_differences: VecDeque::new(),
                store_path,
            }
        }

        pub fn revert(&mut self) -> Result<(), &'static str> {
            let line_difference = self.line_differences.pop_back();
            // TODO: Revert difference
            // Open the path in the line difference
            // Revert line at line number from changed_line to line
            unimplemented!();
        }

        pub fn store(&mut self, line_difference: LineDifference) {
            self.line_differences.push_back(line_difference);
            // TODO: Save to disk
            // Use file at store_path
            // Write line_difference string representation
            unimplemented!();
        }
        pub fn load(&mut self) {
            // TODO: Load from disk
            // Read file at store_path
            // For every line convert a file difference
            unimplemented!();
        }
    }
}