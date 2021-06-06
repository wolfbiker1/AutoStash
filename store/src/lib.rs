pub mod store {
    use diff::LineDifference;
    use itertools::Itertools;
    use pickledb::{error, PickleDb, PickleDbDumpPolicy, SerializationMethod};
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::str::FromStr;
    use walkdir::{DirEntry, WalkDir};

    static CHANGES_QUEUE: &str = "CHANGES_QUEUE";

    pub struct Store {
        db: PickleDb,
    }

    fn version_zero(store_path: &str, watch_path: &str) -> PickleDb {
        let mut db = PickleDb::new(
            store_path,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        );

        db.lcreate(CHANGES_QUEUE)
            .expect("could not create change queue");

        store_all_files(watch_path, &mut db);

        db
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

        pub fn revert(&mut self) -> Result<Option<LineDifference>, &str> {
            // TODO: check if this is necessary (finding the latest file change)
            let change_queue = self.change_queue();
            let last = change_queue.last().ok_or_else(|| "")?;
            let last_pos = self
                .all_by_key(CHANGES_QUEUE)
                .iter()
                .find_position(|line| {
                    line.path.eq(last.path.as_str()) && line.date_time.eq(last.date_time.as_str())
                })
                .map(|(pos, _)| pos)
                .ok_or_else(|| "")?;

            Ok(self.db.lpop(CHANGES_QUEUE, last_pos))
        }

        pub fn store(&mut self, line_difference: &LineDifference) {
            self.db
                .ladd(line_difference.path.as_str(), &line_difference);
            self.db.ladd(CHANGES_QUEUE, &line_difference.path);
        }

        fn change_queue(&mut self) -> Vec<LineDifference> {
            // TODO: use chrone date time for comparison
            self.all_by_key(CHANGES_QUEUE)
                .iter()
                .sorted_by(|a, b| Ord::cmp(a.date_time.as_str(), b.date_time.as_str()))
                .map(|line| line.clone())
                .collect()
        }

        pub fn all_by_key(&self, path: &str) -> Vec<LineDifference> {
            self.db
                .liter(path)
                .map(|line| line.get_item().unwrap())
                .collect()
        }
    }
}
