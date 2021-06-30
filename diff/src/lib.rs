use chrono::{NaiveDateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead};

pub static RFC3339: &str = "%Y-%m-%dT%H:%M:%S%.9f%:z";

/*
    SCENARIOS:
    - Line is changed from 'a' to 'b'
        - Simple: Create a new LineDifference with the same line_number, line set to 'a' and changed_line set to 'b' 
    - Line is moved from line x to line y
        - Tricky: Create a new LineDifference with the new line_number, line set to '' and changed_line set to 'a'
        - If line is moved downwards, decrement all line_numbers of previous lines
        - If line is moved upwards, increment all line_numbers of subsequent lines
    - Line is removed
*/

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

pub fn find(
    path: &str,
    prev_changes: &Vec<LineDifference>,
) -> Result<Vec<LineDifference>, Box<dyn std::error::Error>> {
    let prev_changes = &unique_prev_changes(prev_changes);
    let mut changed_or_added_lines = find_changed_or_added_lines(path, prev_changes)?;
    let line_count = line_count(path)?;

    if has_removed_lines(prev_changes, line_count) {
        changed_or_added_lines = [
            changed_or_added_lines,
            find_removed_lines(prev_changes, line_count),
        ]
        .concat();
    }

    Ok(changed_or_added_lines)
}

fn line_count(path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    Ok(io::BufReader::new(&file).lines().count())
}

fn has_removed_lines(prev_changes: &Vec<LineDifference>, line_count: usize) -> bool {
    prev_changes.len() > line_count
}

fn unique_prev_changes(prev_changes: &Vec<LineDifference>) -> Vec<LineDifference> {
    prev_changes
        .into_iter()
        .sorted_by(|a, b| sort(b.date_time.as_str(), a.date_time.as_str()))
        .dedup_by(|a, b| a.line_number.eq(&b.line_number))
        .sorted_by(|a, b| sort(a.date_time.as_str(), b.date_time.as_str()))
        .map(|e| e.clone())
        .collect_vec()
}

pub fn sort(date_time_a: &str, date_time_b: &str) -> std::cmp::Ordering {
    Ord::cmp(
        &NaiveDateTime::parse_from_str(date_time_a, RFC3339).unwrap(),
        &NaiveDateTime::parse_from_str(date_time_b, RFC3339).unwrap(),
    )
}

fn find_removed_lines(
    prev_changes: &Vec<LineDifference>,
    line_count: usize,
) -> Vec<LineDifference> {
    prev_changes
        .split_at(line_count)
        .1
        .iter()
        .map(|line| {
            LineDifference::new(
                line.path.to_string(),
                line.line_number,
                line.changed_line.to_string(),
                "".to_string(),
            )
        })
        .collect()
}

fn find_changed_or_added_lines(
    path: &str,
    prev_changes: &Vec<LineDifference>,
) -> Result<Vec<LineDifference>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let token = LineDifference::token();

    Ok(io::BufReader::new(file)
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
        .collect())
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
    use std::{fs::{OpenOptions, remove_file}, io::Write};

    fn init(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create(path)?;
        file.write_all("Hello World\n".repeat(5).as_bytes())?;
        Ok(())
    }

    fn read(path: &str) -> Result<Vec<LineDifference>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let res = io::BufReader::new(&file)
            .lines()
            .enumerate()
            .map(|(index, line)| {
                // TODO: Check
                let line = line.unwrap();

                LineDifference::new(path.to_string(), index, "".to_string(), line)
            })
            .collect();

        Ok([
            res,
            vec![
                LineDifference::new(
                    path.to_string(),
                    4,
                    "Hello World".to_string(),
                    "".to_string(),
                ),
                LineDifference::new(
                    path.to_string(),
                    4,
                    "".to_string(),
                    "Hello World".to_string(),
                ),
            ],
        ]
        .concat())
    }

    fn remove(path: &str) -> std::io::Result<()> {
        remove_file(path)
    }
    #[test]
    fn no_changes() {
        let path = "test.txt";
        init(path).unwrap();
        let changes = read(path).unwrap();

        let new_changes: Vec<LineDifference> = find(path, &changes).unwrap();
        remove(path).unwrap();

        assert_eq!(new_changes, []);
    }

    #[test]
    fn changes() {
        let path = "test2.txt";
        init(path).unwrap();
        let changes = read(path).unwrap();

        let mut file = OpenOptions::new().write(true).open(path).unwrap();
        let mut new_file_content: Vec<String> = unique_prev_changes(&changes)
            .iter()
            .map(|line| line.changed_line.to_string() + "\n")
            .collect();
        new_file_content[3] = "Hello W0rld\n".to_string();

        new_file_content.iter().for_each(|line| {
            file.write_all(line.as_bytes())
                .expect("Couldn't write to file.");
        });

        let new_changes: Vec<LineDifference> = find(path, &changes).unwrap();
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
            unique_prev_changes(&changes)
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

        let new_changes: Vec<LineDifference> = find(path, &changes).unwrap();
        remove(path).unwrap();

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                path.to_string(),
                5,
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
                5,
                "".to_string(),
                "Hello World".to_string(),
            )],
        ]
        .concat();

        let new_changes: Vec<LineDifference> = find(path, &changes).unwrap();
        remove(path).unwrap();

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                path.to_string(),
                5,
                "Hello World".to_string(),
                "".to_string()
            )]
        );
    }

    #[test]
    fn should_compare_only_the_latest_prev_changes() {
        let path = "test5.txt";

        let prev_change = LineDifference::new(
            path.to_string(),
            0,
            "".to_string(),
            "Hello World".to_string(),
        );
        let changes: Vec<LineDifference> = vec![
            prev_change,
            LineDifference::new(
                path.to_string(),
                0,
                "Hello World".to_string(),
                "Hello World2".to_string(),
            ),
        ];
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all("Hello World2\nNew Change\n".as_bytes())
            .unwrap();

        let new_changes: Vec<LineDifference> = find(path, &changes).unwrap();
        remove(path).unwrap();

        assert_eq!(
            new_changes,
            vec![LineDifference::new(
                path.to_string(),
                1,
                "".to_string(),
                "New Change".to_string()
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
