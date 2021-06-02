pub mod diff {
    use difference::{Changeset, Difference};    
    use std::error::Error;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct LineDifference {
        pub path: String,
        pub line_number: i32,
        pub line: String,
        pub changed_line: String,
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
