pub mod diff {
    use chrono::Utc;
    use serde::{Deserialize, Serialize};
    use std::fs::File;
    use std::io::{self, BufRead};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct LineDifference {
        pub path: String,
        pub line_number: usize,
        pub line: String,
        pub changed_line: String,
        pub date_time: String,
    }

    impl LineDifference {
        pub fn new(
            path: String,
            line_number: usize,
            line: String,
            changed_line: String,
        ) -> LineDifference {
            LineDifference {
                path,
                line_number,
                line,
                changed_line,
                date_time: Utc::now().to_rfc3339(),
            }
        }
    }

    pub fn find(path: String, line_differences: Vec<LineDifference>) {
        let file = File::open(path).unwrap();
        let found_change = io::BufReader::new(file)
            .lines()
            .enumerate()
            .filter(|(index, line)| {
                line_differences
                    .iter()
                    .find(|line| line.line_number.eq(&index))
                    .is_some()
            });
        /*

            match found_line {
                    Some(found_line) => found_line.changed_line.ne(line.unwrap().as_str()),
                    None => false,
                }
        */
    }
}
