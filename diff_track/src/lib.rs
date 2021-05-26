pub mod diff_tracker {
    use regex::Regex;
    use std::fs::File;
    use std::io::prelude::*;

    #[derive(Debug)]
    pub struct DIFF {
        line_number: i32,
        reverted: bool,
        original_line: String,
        new_line: String,
    }

    impl DIFF {
        pub fn new(
            line_number: i32,
            reverted: bool,
            original_line: String,
            new_line: String,
        ) -> DIFF {
            DIFF {
                line_number,
                reverted,
                original_line,
                new_line,
            }
        }
    }

    pub fn search_diffs<'a>(
        original_line: &str,
        new_line: &str,
        current_line: i32,
        diff_container: &'a mut Vec<DIFF>,
        is_reversed: bool,
    ) -> i32 {
        if original_line != new_line {
            let re_orig = Regex::new(original_line).unwrap();
            let re_new = Regex::new(original_line).unwrap();
            if re_orig.is_match(new_line) || re_new.is_match(original_line) {
                diff_container.push(DIFF::new(
                    current_line,
                    is_reversed,
                    original_line.to_string(),
                    new_line.to_string(),
                ));
                return current_line;
            }
        }
        current_line as i32
    }

    pub fn create_diff_string(diff_container: Vec<DIFF>) -> String {
        // let mut file = File::create("./diffs.txt").unwrap();

        let mut diff_to_write: String = String::new();
        for diff in &diff_container {
            diff_to_write.push_str(&(diff.line_number).to_string());
            diff_to_write.push_str("#=#");
            if diff.reverted {
                if diff.new_line.is_empty() {
                    diff_to_write.push_str("EMPTY");
                } else {
                    diff_to_write.push_str(&diff.new_line);
                }
                diff_to_write.push_str("#=#");

                if diff.original_line.is_empty() {
                    diff_to_write.push_str("EMPTY");
                } else {
                    diff_to_write.push_str(&diff.original_line);
                }
                diff_to_write.push_str("#=#\n");
            } else {
                if diff.original_line.is_empty() {
                    diff_to_write.push_str("EMPTY");
                } else {
                    diff_to_write.push_str(&diff.original_line);
                }
                diff_to_write.push_str("#=#");

                if diff.new_line.is_empty() {
                    diff_to_write.push_str("EMPTY");
                } else {
                    diff_to_write.push_str(&diff.new_line);
                }
                diff_to_write.push_str("#=#\n");
            }
            // file.write_all(diff_to_write.as_bytes());
        }
        diff_to_write
    }

    pub fn collect_diffs(
        orig: &[std::result::Result<std::string::String, std::io::Error>],
        edited_file_content: &[std::result::Result<std::string::String, std::io::Error>],
        is_reversed: bool,
    ) -> Vec<DIFF> {
        let mut diff_container: Vec<DIFF> = Vec::new();

        let mut indices_to_cover: Vec<i32> = (0..edited_file_content.len() as i32).collect();
        for line_number in 0..orig.len() {
            let line_orig = orig[line_number].as_ref().unwrap();
            let line_new = edited_file_content[line_number].as_ref().unwrap();
            let covered_line: i32 = search_diffs(
                line_orig,
                line_new,
                line_number as i32,
                &mut diff_container,
                is_reversed,
            );
            let index: Option<usize> = indices_to_cover.iter().position(|x| *x == covered_line);
            match index {
                Some(res) => {
                    indices_to_cover.remove(res);
                }
                None => {}
            }
        }

        // when new lines were added
        for line_number in indices_to_cover {
            diff_container.push(DIFF::new(
                line_number,
                is_reversed,
                String::from(""),
                edited_file_content[line_number as usize]
                    .as_ref()
                    .unwrap()
                    .clone(),
            ));
        }
        diff_container
    }
}
