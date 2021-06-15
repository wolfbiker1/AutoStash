extern crate simple_error;

pub mod store {
    use chrono::Utc;
    use chrono::{NaiveDateTime, NaiveTime};
    use diff::LineDifference;
    use itertools::Itertools;
    use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
    use serde::de::DeserializeOwned;
    use std::error;
    use std::fs::File;
    use std::io::Write;
    use std::io::{self, BufRead};
    use walkdir::{DirEntry, WalkDir};

    static CHANGE_PEEK_STACK: &str = "CHANGE_PEEK_STACK";
    static CHANGE_MARKER: &str = "CHANGE_MARKER";
    pub struct Store {
        db: PickleDb,
        pub time_slot: TimeSlot,
    }

    #[derive(Clone)]
    pub struct Version {
        pub name: String,
        pub datetime: NaiveDateTime,
        pub changes: Vec<LineDifference>,
    }

    pub enum TimeSlot {
        HOUR,
        DAY,
        WEEK,
    }

    impl TimeSlot {
        pub fn value(&self) -> u32 {
            match &self {
                &Self::HOUR => 1 * 60 * 60,
                &Self::DAY => 24 * &Self::HOUR.value(),
                &Self::WEEK => 7 * &Self::DAY.value(),
            }
        }
    }

    fn version_zero(store_path: &str, watch_path: &str) -> Result<PickleDb, Box<dyn error::Error>> {
        let mut db = PickleDb::new(
            store_path,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Yaml,
        );

        init_change_stack(&mut db)?;
        store_all_files(watch_path, &mut db)?;

        Ok(db)
    }

    fn init_change_stack(db: &mut PickleDb) -> Result<(), Box<dyn error::Error>> {
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
                time_slot: TimeSlot::HOUR,
            })
        }

        pub fn undo_by(&mut self, count: usize) -> Result<(), Box<dyn error::Error>> {
            self.undo(count, true)
        }

        pub fn redo_by(&mut self, count: usize) -> Result<(), Box<dyn error::Error>> {
            self.undo(count, false)
        }

        pub fn view(&mut self) -> Result<Vec<Version>, Box<dyn error::Error>> {
            let marker = self.get_change_marker()?;
            let now = Utc::now().naive_utc();

            Ok(self
                .db
                .liter(CHANGE_PEEK_STACK)
                .take(marker)
                .map(|e| -> String { e.get_item().unwrap() })
                .dedup_by(|a, b| a.eq(b))
                .enumerate()
                .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
                .map(|f| -> Version {
                    let path = f.1;
                    let mut earliest_datetime = now;
                    let changes: Vec<LineDifference> = self
                        .get_changes::<LineDifference>(path.as_str())
                        .iter()
                        .sorted_by(|a, b| diff::sort(b.date_time.as_str(), a.date_time.as_str()))
                        .take_while(|e| {
                            // TODO: Propagate error
                            let datetime =
                                NaiveDateTime::parse_from_str(e.date_time.as_str(), diff::RFC3339)
                                    .unwrap();
                            if earliest_datetime.timestamp() > datetime.timestamp() {
                                earliest_datetime = datetime;
                            }
                            let time_frame = i64::from(self.time_slot.value());
                            now.timestamp() - time_frame < datetime.timestamp()
                        })
                        .map(|e| e.clone())
                        .collect_vec();

                    Version {
                        name: path,
                        datetime: earliest_datetime,
                        changes,
                    }
                })
                .collect_vec())
        }

        pub fn get_changes<T: DeserializeOwned + std::fmt::Debug>(&self, path: &str) -> Vec<T> {
            self.db
                .liter(path)
                .map(|e| e.get_item::<T>().unwrap())
                .collect()
        }

        pub fn store_changes(
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
            let peek_stack_length = self.get_changes::<String>(CHANGE_PEEK_STACK).len();

            self.set_change_marker(&peek_stack_length)
        }

        fn undo(&mut self, count: usize, previous: bool) -> Result<(), Box<dyn error::Error>> {
            // Peek stack backwards by count
            // Undo line differences on the peeked files for certain versioning
            // Decrement change marker by count

            let file_stack = self.peek_files(count, previous)?;
            self.undo_files(file_stack)
            //self.decrement_change_marker_by(count)
        }

        fn peek_files(
            &mut self,
            count: usize,
            previous: bool,
        ) -> Result<Vec<String>, Box<dyn error::Error>> {
            let marker = self.get_change_marker()?;
            let (begin, end);
            if previous {
                begin = marker - count;
                end = marker;
            } else {
                begin = marker;
                end = marker + count;
            }

            Ok(self
                .db
                .liter(CHANGE_PEEK_STACK)
                .skip(begin)
                .take(end)
                .map(|e| e.get_item::<String>().unwrap())
                .dedup_by(|a, b| a.eq(b))
                .collect())
        }

        fn undo_files(&self, file_stack: Vec<String>) -> Result<(), Box<dyn error::Error>> {
            file_stack.iter().for_each(|file| {
                let changed_lines = self
                    .get_changes::<LineDifference>(file)
                    .iter()
                    .sorted_by(|a, b| diff::sort(b.date_time.as_str(), a.date_time.as_str()))
                    .map(|e| e.clone())
                    .collect();
                // TODO: propagate error
                self.undo_changes(self.restrict_by_time_slot(changed_lines))
                    .unwrap()
            });
            Ok(())
        }

        fn undo_changes(
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
            let limit_time = latest_time.time() - NaiveTime::from_hms(self.time_slot.value(), 0, 0);

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
