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

pub fn find_new_changes(
    path: &str,
    prev_changes: &Vec<LineDifference>,
) -> Result<Vec<LineDifference>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let changed_or_added_lines = find_changed_or_added_lines(&file, path, prev_changes);
    let file = File::open(path)?;
    let lines = io::BufReader::new(&file).lines().count();

    if has_removed_lines(prev_changes, lines) {
        return Ok([
            changed_or_added_lines,
            find_removed_lines(prev_changes, lines),
        ]
        .concat());
    }

    Ok(changed_or_added_lines)
}

fn has_removed_lines(prev_changes: &Vec<LineDifference>, line_count: usize) -> bool {
    prev_changes.len() > line_count
}

fn find_removed_lines(
    prev_changes: &Vec<LineDifference>,
    line_count: usize,
) -> Vec<LineDifference> {
    // TODO: Sort by time
    prev_changes
        .split_at(line_count)
        .1
        .iter()
        .enumerate()
        .map(|(index, line)| {
            LineDifference::new(
                line.path.to_string(),
                line_count + index,
                line.changed_line.to_string(),
                "".to_string(),
            )
        })
        .collect()
}

fn find_changed_or_added_lines(
    file: &File,
    path: &str,
    prev_changes: &Vec<LineDifference>,
) -> Vec<LineDifference> {
    let token = LineDifference::token();

    io::BufReader::new(file)
        .lines()
        .enumerate()
        .map(|(index, line)| {
            // TODO: Check
            if line.is_err() {
                return token.clone();
            }

            find_changed_or_added_line(prev_changes, index, path, line.unwrap(), &token)
        })
        .filter(|e| e.ne(&token))
        .collect()
}

fn find_changed_or_added_line(
    prev_changes: &Vec<LineDifference>,
    index: usize,
    path: &str,
    line: String,
    token: &LineDifference,
) -> LineDifference {
    let found_change = prev_changes
        .iter()
        .find(|found| found.line_number.eq(&index));

    if found_change.is_some() {
        let found_change = found_change.unwrap();
        if found_change.changed_line.ne(&line) {
            // changed line
            return LineDifference::new(
                path.to_string(),
                index,
                found_change.changed_line.clone(),
                line,
            );
        }
        // unchanged line
        return token.clone();
    }

    // added line
    LineDifference::new(path.to_string(), index, "".to_string(), line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;

    fn init(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create(path)?;
        file.write_all("Hello World\n".repeat(10).as_bytes())?;
        Ok(())
    }

    fn read(path: &str) -> Result<Vec<LineDifference>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        io::BufReader::new(&file)
            .lines()
            .enumerate()
            .map(|(index, line)| {
                let line = line?;

                Ok(LineDifference::new(
                    path.to_string(),
                    index,
                    "".to_string(),
                    line,
                ))
            })
            .collect()
    }

    fn remove(path: &str) -> std::io::Result<()> {
        remove_file(path)
    }
    #[test]
    fn no_changes() {
        let path = "test.txt";
        init(path).unwrap();
        let changes = read(path).unwrap();

        let new_changes: Vec<LineDifference> = find_new_changes(path, &changes).unwrap();
        remove(path).unwrap();

        assert_eq!(new_changes, []);
    }

    #[test]
    fn changes() {
        let path = "test2.txt";
        init(path).unwrap();
        let changes = read(path).unwrap();

        let mut file = OpenOptions::new().write(true).open(path).unwrap();
        let mut new_file_content: Vec<String> = changes
            .iter()
            .map(|line| line.changed_line.to_string() + "\n")
            .collect();
        new_file_content[3] = "Hello W0rld\n".to_string();

        new_file_content.iter().for_each(|line| {
            file.write_all(line.as_bytes())
                .expect("Couldn't write to file.");
        });

        let new_changes: Vec<LineDifference> = find_new_changes(path, &changes).unwrap();
        remove(path).unwrap();

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                path.to_string(),
                3,
                "Hello World".to_string(),
                "Hello W0rld".to_string()
            )]
        );
    }

    #[test]
    fn more_lines_than_differences() {
        let path = "test3.txt";
        init(path).unwrap();
        let changes = read(path).unwrap();

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

        let new_changes: Vec<LineDifference> = find_new_changes(path, &changes).unwrap();
        remove(path).unwrap();

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                path.to_string(),
                10,
                "".to_string(),
                "Hello World".to_string()
            )]
        );
    }

    #[test]
    fn more_differences_than_lines() {
        let path = "test4.txt";
        init(path).unwrap();

        let changes: Vec<LineDifference> = [
            read(path).unwrap(),
            vec![LineDifference::new(
                path.to_string(),
                10,
                "".to_string(),
                "Hello World".to_string(),
            )],
        ]
        .concat();

        let new_changes: Vec<LineDifference> = find_new_changes(path, &changes).unwrap();
        remove(path).unwrap();

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                path.to_string(),
                10,
                "Hello World".to_string(),
                "".to_string()
            )]
        );
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
