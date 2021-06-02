pub mod diff {
    use difference::{Changeset, Difference};    
    use std::error::Error;

    pub struct LineDifference {
        path: String,
        line_number: i32,
        line: String,
        changed_line: String,
    }

    impl LineDifference {
        pub fn new(
            path: String,
            line_number: i32,
            line: String,
            changed_line: String,
        ) -> LineDifference {
            LineDifference {
                path,
                line_number,
                line,
                changed_line,
            }
        }

        pub fn to_stored_line_difference(&self) -> String {
            // TODO: Serialization
            unimplemented!();
        }

        pub fn to_line_difference(stored_line_difference: String) -> LineDifference {
            // TODO: Deserialization
            unimplemented!();
        }
    }

    pub fn find(path: String) -> Result<LineDifference, &'static str> {
        // TODO:
        // Compare every line
        // let changeset = Changeset::new(line, changed_line, "");
        // Comparison: Difference::Same("") | Difference::Rem("") | Difference::Add("")
        // Max(path, other_path) #lines
        unimplemented!();
    }
}
