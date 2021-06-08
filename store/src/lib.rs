extern crate simple_error;

pub mod store {
    use chrono::{NaiveDateTime, NaiveTime};
    use diff::LineDifference;
    use itertools::Itertools;
    use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
    use serde::de::DeserializeOwned;
    use simple_error::SimpleError;
    use std::collections::VecDeque;
    use std::error;
    use std::fs::File;
    use std::io::Write;
    use std::io::{self, BufRead};
    use std::str::FromStr;
    use walkdir::{DirEntry, WalkDir};

    static CHANGE_PEEK_STACK: &str = "CHANGE_PEEK_STACK";
    static CHANGE_MARKER: &str = "CHANGE_MARKER";
    pub struct Store {
        db: PickleDb,
        time_slot_hours: u32,
    }

    fn version_zero(store_path: &str, watch_path: &str) -> Result<PickleDb, Box<dyn error::Error>> {
        let mut db = PickleDb::new(
            store_path,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Yaml,
        );

        create_change_stack(&mut db)?;
        store_all_files(watch_path, &mut db)?;

        Ok(db)
    }

    fn create_change_stack(db: &mut PickleDb) -> Result<(), Box<dyn error::Error>> {
        db.lcreate(CHANGE_PEEK_STACK)?;
        db.lcreate(CHANGE_MARKER)?;
        db.set(CHANGE_MARKER, &0).map_err(|err| err.into())
    }

    fn store_all_files(watch_path: &str, db: &mut PickleDb) -> Result<(), Box<dyn error::Error>> {
        WalkDir::new(watch_path)
            .into_iter()
            .filter(|entry| match entry {
                Ok(entry) => entry.path().is_file(),
                _ => true,
            })
            .map(
                |entry: Result<DirEntry, walkdir::Error>| -> Result<(), Box<dyn error::Error>> {
                    let entry = entry?;
                    let path = entry.path();
                    let p = path.to_str().unwrap_or_else(|| "couldn't find path");
                    db.lcreate(p)?;
                    store_lines_from_file(p.to_string(), db)
                },
            )
            .find(|e| e.is_err())
            .unwrap_or_else(|| Ok(()))
    }

    fn store_lines_from_file(path: String, db: &mut PickleDb) -> Result<(), Box<dyn error::Error>> {
        let file = File::open(path.clone())?;
        io::BufReader::new(file)
            .lines()
            .enumerate()
            .map(|(index, line)| {
                db.ladd(
                    path.as_str(),
                    &LineDifference::new(path.clone(), index, "".to_string(), line.unwrap()),
                )
                .map(|_| ())
                .ok_or_else(|| "couldn't add line difference".into())
            })
            .find(|e| e.is_err())
            .unwrap_or_else(|| Ok(()))
    }

    fn load(store_path: &str) -> Result<PickleDb, Box<dyn error::Error>> {
        PickleDb::load(
            store_path,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Yaml,
        )
        .map_err(|err| err.into())
    }

    impl Store {
        pub fn new(store_path: &str, watch_path: &str) -> Result<Store, Box<dyn error::Error>> {
            let mut db = load(store_path);
            if db.is_err() {
                db = version_zero(store_path, watch_path);
            }
            let db = db?;

            Ok(Store {
                db,
                time_slot_hours: 1,
            })
        }

        pub fn undo_by(&mut self, count: usize, previous: bool) -> Result<(), Box<dyn error::Error>> {
            // Peek stack backwards by count
            // Undo line differences on the peeked files for certain versioning
            // Decrement change marker by count

            let file_stack = self.peek_files(count, previous);
            self.undo_files(file_stack)
            //self.decrement_change_marker_by(count)
        }

        pub fn undo(&mut self) -> Result<(), Box<dyn error::Error>> {
            self.undo_by(1, true)
        }

        pub fn redo_by(&mut self, count: usize) -> Result<(), Box<dyn error::Error>> {
            self.undo_by(count, false)
        }

        pub fn redo(&mut self) -> Result<(), Box<dyn error::Error>> {
            self.redo_by(1)
        }

        pub fn get_differences_by_path<T: DeserializeOwned + std::fmt::Debug>(
            &self,
            path: &str,
        ) -> Vec<T> {
            self.db
                .liter(path)
                .map(|e| e.get_item::<T>().unwrap())
                .collect()
        }

        pub fn store_all_differences(
            &mut self,
            path: &str,
            changes: &Vec<LineDifference>,
        ) -> Result<(), Box<dyn error::Error>> {
            if !self.db.lexists(path) {
                self.db.lcreate(path)?;
            }
            self.db.lextend(path, changes);
            self.db
                .ladd(CHANGE_PEEK_STACK, &path.to_string())
                .ok_or_else(|| "couldn't add file path to change peek stack")?;
            let peek_stack_length = self
                .get_differences_by_path::<String>(CHANGE_PEEK_STACK)
                .len();

            self.set_change_marker(&peek_stack_length)
        }

        fn peek_files(&mut self, count: usize, previous: bool) -> VecDeque<String> {
            // TODO: time slot peek
            let mut peeked: VecDeque<String> = VecDeque::new();
            let marker = self.stack_marker();
            let (begin, end);
            if previous {
                begin = marker - count;
                end = marker;
            } else {
                begin = marker;
                end = marker + count;
            }
            //TODO: Fix
            for i in begin..end {
                peeked.push_front(self.db.lget(CHANGE_PEEK_STACK, i).unwrap());
            }
            peeked
                .iter()
                .dedup_by(|a, b| a.eq(b))
                .map(|e| e.clone())
                .collect()
        }

        fn undo_files(&self, file_stack: VecDeque<String>) -> Result<(), Box<dyn error::Error>> {
            file_stack.iter().for_each(|file| {
                let changed_lines = self
                    .get_differences_by_path::<LineDifference>(file)
                    .iter()
                    .sorted_by(|a, b| diff::sort(b.date_time.as_str(), a.date_time.as_str()))
                    .map(|e| e.clone())
                    .collect();
                // TODO: propagate error
                self.undo_line_differences(self.restrict_by_time_slot(changed_lines)).unwrap()
            });
            Ok(())
        }

        fn undo_line_differences(
            &self,
            changed_lines: Vec<LineDifference>,
        ) -> Result<(), Box<dyn error::Error>> {
            let path = changed_lines[0].path.clone();
            let file = File::open(path.clone())?;
            let reverted_lines: Vec<String> = io::BufReader::new(file)
                .lines()
                .map(|l| l.unwrap())
                .enumerate()
                .map(|(index, line)| {
                    let found = changed_lines.iter().find(|l| l.line_number.eq(&index));
                    if found.is_some() {
                        let found = found.unwrap();
                        if found.line.eq(&line) {
                            return found.changed_line.clone();
                        }

                        return found.line.clone();
                    }
                    return line;
                })
                .collect();

            let mut file = File::open(path.clone())?;
            file.write_all(reverted_lines.join("").as_bytes())
                .map_err(|err| err.into())
        }

        fn restrict_by_time_slot(&self, changed_lines: Vec<LineDifference>) -> Vec<LineDifference> {
            let latest_time = NaiveDateTime::parse_from_str(
                changed_lines.last().unwrap().date_time.as_str(),
                diff::RFC3339,
            )
            .unwrap();
            let limit_time = latest_time.time() - NaiveTime::from_hms(self.time_slot_hours, 0, 0);

            changed_lines
                .iter()
                .filter(|line| {
                    let time =
                        NaiveDateTime::parse_from_str(line.date_time.as_str(), diff::RFC3339)
                            .unwrap();
                    latest_time - time <= limit_time
                })
                .map(|e| e.clone())
                .collect()
        }

        fn stack_marker(&mut self) -> usize {
            self.db.get(CHANGE_MARKER).unwrap()
        }

        

        fn increment_change_marker_by(
            &mut self,
            count: usize,
        ) -> Result<(), Box<dyn error::Error>> {
            self.set_change_marker(&(self.get_change_marker()? + count))
        }

        fn decrement_change_marker_by(
            &mut self,
            count: usize,
        ) -> Result<(), Box<dyn error::Error>> {
            self.set_change_marker(&(self.get_change_marker()? - count))
        }

        fn set_change_marker(&mut self, count: &usize) -> Result<(), Box<dyn error::Error>> {
            self.db.set(CHANGE_MARKER, count).map_err(|err| err.into())
        }

        fn get_change_marker(&self) -> Result<usize, Box<dyn error::Error>> {
            self.db
                .get::<usize>(CHANGE_MARKER)
                .ok_or_else(|| "couln't get change marker".into())
        }
    }
}
