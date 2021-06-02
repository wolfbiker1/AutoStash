pub mod store {
    use diff::diff::LineDifference;
    use itertools::Itertools;
    use pickledb::{error, PickleDb, PickleDbDumpPolicy, SerializationMethod};
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::str::FromStr;
    use walkdir::{DirEntry, WalkDir};

    static LAST_EDITED_FILE: &str = "LAST_EDITED_FILE";

    pub struct Store {
        db: PickleDb,
    }

    fn version_zero(store_path: &str, watch_path: &str) -> Result<PickleDb, std::io::Error> {
        let mut db = PickleDb::new(
            store_path,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        );

        store_all_files(watch_path, &mut db);

        Ok(db)
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
                    line.clone(),
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
        pub fn new(store_path: &str, watch_path: &str) -> Result<Store, std::io::Error> {
            let db = match load(store_path) {
                Ok(db) => Ok(db),
                Err(_) => version_zero(store_path, watch_path),
            };

            match db {
                Err(e) => Err(e),
                Ok(db) => Ok(Store { db }),
            }
        }

        pub fn revert(&mut self) -> Result<Option<LineDifference>, &str> {
            if !self.db.exists(LAST_EDITED_FILE) {
                return Err("");
            }
            let last_edited_file = self
                .last_edited_file()
                .ok_or_else(|| "No last edited file found")?;

            // TODO: check if this is necessary (finding the latest file change)
            let last = self
                .by_file(last_edited_file.as_str())
                .into_iter()
                // TODO: use chrone date time for comparison
                .sorted_by(|a, b| Ord::cmp(a.date_time.as_str(), b.date_time.as_str()))
                .len()
                - 1;

            Ok(self.db.lpop(last_edited_file.as_str(), last))
        }

        pub fn store(&mut self, line_difference: &LineDifference) -> Result<(), error::Error> {
            self.db
                .ladd(line_difference.path.as_str(), &line_difference);
            self.db.set(LAST_EDITED_FILE, &line_difference.path)
        }

        fn last_edited_file(&mut self) -> Option<String> {
            self.db.get(LAST_EDITED_FILE)
        }

        pub fn by_file(&self, path: &str) -> Vec<LineDifference> {
            self.db
                .liter(path)
                .map(|line| line.get_item().unwrap())
                .collect()
        }
    }
}
