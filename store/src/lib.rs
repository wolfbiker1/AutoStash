extern crate simple_error;

pub mod store {
    use diff::LineDifference;
    use itertools::Itertools;
    use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
    use serde::de::DeserializeOwned;
    use simple_error::SimpleError;
    use std::error;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::str::FromStr;
    use walkdir::{DirEntry, WalkDir};

    static CHANGE_PEEK_STACK: &str = "CHANGE_PEEK_STACK";
    static CHANGE_MARKER: &str = "CHANGE_MARKER";
    pub struct Store {
        db: PickleDb,
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

            Ok(Store { db })
        }

        pub fn undo_by(&mut self, count: usize) {
            // Peek stack backwards by count
            // Undo line differences on the peeked files for certain versioning
            // Decrement change marker by count
        }

        pub fn undo(&mut self) {
            self.undo_by(1)
        }

        pub fn redo_by(&mut self, count: usize) {
            // Peek stack forwards by count
            // Redo line differences on the peeked files for certain versioning
            // Increment change marker by count
        }

        pub fn redo(&mut self) {
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
