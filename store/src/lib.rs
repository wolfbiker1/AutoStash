pub mod store {
    use diff::LineDifference;
    use itertools::Itertools;
    use pickledb::error::Error;
    use pickledb::{error, PickleDb, PickleDbDumpPolicy, SerializationMethod};
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::str::FromStr;
    use walkdir::{DirEntry, WalkDir};

    static CHANGE_PEEK_STACK: &str = "CHANGE_PEEK_STACK";
    static CHANGE_MARKER: &str = "CHANGE_MARKER";

    pub struct Store {
        db: PickleDb,
    }

    fn version_zero(store_path: &str, watch_path: &str) -> PickleDb {
        let mut db = PickleDb::new(
            store_path,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        );

        create_change_stack(&mut db);
        store_all_files(watch_path, &mut db);

        db
    }

    fn create_change_stack(db: &mut PickleDb) {
        db.lcreate(CHANGE_PEEK_STACK)
            .expect("could not create change peek stack");

        db.lcreate(CHANGE_MARKER)
            .expect("could not create change marker");

        db.set(CHANGE_MARKER, &0)
            .unwrap_or_else(|err| panic!("could not set change marker: {}", err));
    }

    fn store_all_files(watch_path: &str, db: &mut PickleDb) {
        WalkDir::new(watch_path)
            .into_iter()
            .for_each(|entry: Result<DirEntry, walkdir::Error>| {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() {
                    match db.lcreate(path.to_str().unwrap()) {
                        Err(e) => eprintln!("error during db file list creation {:?}", e),
                        _ => (),
                    }

                    match store_lines_from_file(path, db) {
                        Err(e) => eprintln!("error during lines from file storage {:?}", e),
                        _ => (),
                    }
                }
            })
    }

    fn store_lines_from_file(
        path: &std::path::Path,
        db: &mut PickleDb,
    ) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let path = path.to_str().unwrap();
        io::BufReader::new(file)
            .lines()
            .enumerate()
            .map(|indexed_line| -> Result<LineDifference, std::io::Error> {
                let line = indexed_line.1?;
                Ok(LineDifference::new(
                    String::from_str(path).unwrap(),
                    indexed_line.0,
                    "".to_string(),
                    line,
                ))
            })
            .for_each(|line| match line {
                Ok(l) => {
                    db.ladd(path, &l);
                }
                Err(e) => {
                    eprintln!("error during file traversal {:?}", e);
                }
            });
        Ok(())
    }

    fn load(store_path: &str) -> Result<PickleDb, error::Error> {
        PickleDb::load(
            store_path,
            PickleDbDumpPolicy::DumpUponRequest,
            SerializationMethod::Json,
        )
    }

    impl Store {
        pub fn new(store_path: &str, watch_path: &str) -> Store {
            let db = match load(store_path) {
                Ok(db) => db,
                Err(_) => version_zero(store_path, watch_path),
            };

            Store { db }
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

        pub fn get_differences_by_path(&self, path: &str) -> Vec<LineDifference> {
            self.db.get(path).unwrap_or_else(|| vec![])
        }

        pub fn store_all_differences(
            &mut self,
            path: &str,
            changes: &Vec<LineDifference>,
        ) -> Result<(), Error> {
            self.db.lextend(path, changes);
            self.db.ladd(CHANGE_PEEK_STACK, &path);
            self.set_change_marker(&self.get_differences_by_path(CHANGE_PEEK_STACK).len())
        }

        fn increment_change_marker_by(&mut self, count: usize) -> Result<(), Error> {
            self.set_change_marker(&(self.get_change_marker().unwrap() + count))
        }

        fn decrement_change_marker_by(&mut self, count: usize) -> Result<(), Error> {
            self.set_change_marker(&(self.get_change_marker().unwrap() - count))
        }

        fn set_change_marker(&mut self, count: &usize) -> Result<(), Error> {
            self.db.set(CHANGE_MARKER, count)
        }

        fn get_change_marker(&self) -> Option<usize> {
            self.db.get::<usize>(CHANGE_MARKER)
        }
    }
}
