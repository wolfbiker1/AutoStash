pub mod diff {
    use std::collections::VecDeque;
    use difference::{Difference, Changeset};
    use std::fs::File;
    use std::io::prelude::*;

    pub struct FileDifference {
        path: String,
        line_number: i32,
        line: String,
        changed_line: String,
    }

    impl FileDifference {
        pub fn new(
            path: String,
            line_number: i32,
            line: String,
            changed_line: String,
        ) -> FileDifference {
            FileDifference {
                path,
                line_number,
                line,
                changed_line,
            }
        }

        pub fn to_string(&self) -> String {
            // TODO: String representation
        }
    }

    pub fn to_file_difference(file_difference: String) -> FileDifference {
        // TODO: File difference conversion
    }

    pub fn find(path: String, other_path: String) -> Result<Err, FileDifference> {
        // TODO: Open both files
        // Compare every line
        // let changeset = Changeset::new(line, changed_line, "");
        // Comparison: Difference::Same("") | Difference::Rem("") | Difference::Add("")
        // Max(path, other_path) #lines 
    }

    pub struct FileDifferenceStore {
        file_differences: VecDeque<FileDifference>,
        store_path: String,
    }

    impl FileDifferenceStore {
        pub fn new(store_path: String) -> FileDifferenceStore {
            FileDifferenceStore { file_differences: VecDeque::new(), store_path }
        }

        pub fn revert(&mut self) -> Result<Err, ()> {
            let file_difference = self.file_differences.pop_back();
            // TODO: Revert difference
            // Open the path in the file difference
            // Revert line at line number from changed_line to line 
        }

        pub fn store(&mut self, file_difference: FileDifference) {
            self.file_differences.push_back(file_difference);
            // TODO: Save to disk
            // Use file at store_path
            // Write file_difference string representation
        }
        pub fn load(&mut self) {
            // TODO: Load from disk
            // Read file at store_path
            // For every line convert a file difference
        }
    }
}
