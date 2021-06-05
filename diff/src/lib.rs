use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead};

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
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

    pub fn token() -> LineDifference {
        LineDifference::new("".to_string(), 0, "".to_string(), "".to_string())
    }
}

impl PartialEq for LineDifference {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
            && self.line_number.eq(&other.line_number)
            && self.line.eq(&other.line)
            && self.changed_line.eq(&other.changed_line)
    }
}

pub fn find(path: &str, line_differences: &Vec<LineDifference>) -> Vec<LineDifference> {
    let file = File::open(path).unwrap();
    let token = LineDifference::token();
    let changes: Vec<LineDifference> = io::BufReader::new(&file)
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let found_line_difference = line_differences
                .iter()
                .find(|found| found.line_number.eq(&(index + 1)));
            let line = line.unwrap().clone();
            if found_line_difference.is_some() {
                // changed line
                let found_line_difference = found_line_difference.unwrap();
                if found_line_difference.changed_line.ne(&line) {
                    return LineDifference::new(
                        path.to_string(),
                        index + 1,
                        found_line_difference.changed_line.clone(),
                        line,
                    );
                }
                return token.clone();
            }
            return LineDifference::new(path.to_string(), index + 1, "".to_string(), line);
        })
        .filter(|line| line.ne(&token))
        .collect();
    let file = File::open(path).unwrap();
    let line_count = io::BufReader::new(&file).lines().count();

    if line_differences.len() > line_count {
        return [
            changes,
            line_differences
                .split_at(line_count)
                .1
                .iter()
                .enumerate()
                .map(|(index, line)| {
                    LineDifference::new(
                        line.path.to_string(),
                        line_count + index + 1,
                        line.changed_line.to_string(),
                        "".to_string(),
                    )
                })
                .collect(),
        ]
        .concat();
    }

    changes
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;

    use super::*;

    #[ctor::ctor]
    fn init() {
        let mut file = std::fs::File::create("data.txt").expect("create failed");
        file.write_all("Hello World\n".repeat(10).as_bytes())
            .expect("write failed");
    }
    #[test]
    fn no_changes() {
        let file = File::open("data.txt").unwrap();
        let changes: Vec<LineDifference> = io::BufReader::new(&file)
            .lines()
            .enumerate()
            .map(|(index, line)| {
                let line = line.unwrap();

                LineDifference::new("data.txt".to_string(), index + 1, line.clone(), line)
            })
            .collect();

        let new_changes: Vec<LineDifference> = find("data.txt", &changes);

        assert_eq!(new_changes, []);
    }

    #[test]
    fn changes() {
        let file = File::open("data.txt").unwrap();
        let changes: Vec<LineDifference> = io::BufReader::new(&file)
            .lines()
            .enumerate()
            .map(|(index, line)| {
                let line = line.unwrap();

                LineDifference::new("data.txt".to_string(), index + 1, line.clone(), line)
            })
            .collect();

        let mut file = OpenOptions::new().write(true).open("data.txt").unwrap();
        let mut new_file_content: Vec<String> = changes
            .iter()
            .map(|line| line.line.to_string() + "\n")
            .collect();
        new_file_content[3] = "Hello W0rld\n".to_string();

        new_file_content.iter().for_each(|line| {
            file.write_all(line.as_bytes())
                .expect("Couldn't write to file.");
        });

        let new_changes: Vec<LineDifference> = find("data.txt", &changes);

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                "data.txt".to_string(),
                4,
                "Hello World".to_string(),
                "Hello W0rld".to_string()
            )]
        );
    }

    #[test]
    fn more_lines_than_differences() {}

    #[test]
    fn more_differences_than_lines() {}
}
