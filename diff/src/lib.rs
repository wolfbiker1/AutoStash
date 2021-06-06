use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{remove_file, File};
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
    use super::*;
    use std::fs::OpenOptions;

    fn init(path: &str) {
        let mut file = std::fs::File::create(path).expect("create failed");
        file.write_all("Hello World\n".repeat(10).as_bytes())
            .expect("write failed");
    }

    fn read(path: &str) -> Vec<LineDifference> {
        let file = File::open(path).unwrap();
        io::BufReader::new(&file)
            .lines()
            .enumerate()
            .map(|(index, line)| {
                let line = line.unwrap();

                LineDifference::new(path.to_string(), index + 1, line.clone(), line)
            })
            .collect()
    }

    fn remove(path: &str) -> std::io::Result<()> {
        remove_file(path)?;
        Ok(())
    }
    #[test]
    fn no_changes() {
        let path = "test.txt";
        init(path);
        let changes = read(path);

        let new_changes: Vec<LineDifference> = find(path, &changes);
        remove(path).unwrap();

        assert_eq!(new_changes, []);
    }

    #[test]
    fn changes() {
        let path = "test2.txt";
        init(path);
        let changes = read(path);

        let mut file = OpenOptions::new().write(true).open(path).unwrap();
        let mut new_file_content: Vec<String> = changes
            .iter()
            .map(|line| line.line.to_string() + "\n")
            .collect();
        new_file_content[3] = "Hello W0rld\n".to_string();

        new_file_content.iter().for_each(|line| {
            file.write_all(line.as_bytes())
                .expect("Couldn't write to file.");
        });

        let new_changes: Vec<LineDifference> = find(path, &changes);
        remove(path).unwrap();

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                path.to_string(),
                4,
                "Hello World".to_string(),
                "Hello W0rld".to_string()
            )]
        );
    }

    #[test]
    fn more_lines_than_differences() {
        let path = "test3.txt";
        init(path);
        let changes = read(path);

        let mut file = OpenOptions::new().write(true).open(path).unwrap();
        let new_file_content = [
            changes
                .iter()
                .map(|line| line.clone().changed_line + "\n")
                .collect(),
            vec!["Hello World\n".to_string()],
        ]
        .concat();

        new_file_content.iter().for_each(|line| {
            file.write_all(line.as_bytes())
                .expect("Couldn't write to file.");
        });

        let new_changes: Vec<LineDifference> = find(path, &changes);
        remove(path).unwrap();

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                path.to_string(),
                11,
                "".to_string(),
                "Hello World".to_string()
            )]
        );
    }

    #[test]
    fn more_differences_than_lines() {
        let path = "test4.txt";
        init(path);

        let changes: Vec<LineDifference> = [
            read(path),
            vec![LineDifference::new(
                path.to_string(),
                11,
                "".to_string(),
                "Hello World".to_string(),
            )],
        ]
        .concat();

        let new_changes: Vec<LineDifference> = find(path, &changes);
        remove(path).unwrap();

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                path.to_string(),
                11,
                "Hello World".to_string(),
                "".to_string()
            )]
        );
    }

    #[ignore]
    #[test]
    fn file_removed() {
        unimplemented!();
    }

    #[ignore]
    #[test]
    fn a_lot_of_lines() {
        unimplemented!();
    }

    #[ignore]
    #[test]
    fn a_lot_of_differences() {
        unimplemented!();
    }
}
