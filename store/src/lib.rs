pub mod store {
    use diff::diff::LineDifference;
    use pickledb::{error, PickleDb, PickleDbDumpPolicy, SerializationMethod};
    use std::collections::VecDeque;

    pub struct Store {
        line_differences: VecDeque<LineDifference>,
        db: PickleDb,
    }

    impl Store {
        pub fn new(store_path: String) -> Store {
            let db = match PickleDb::load(
                store_path.clone(),
                PickleDbDumpPolicy::DumpUponRequest,
                SerializationMethod::Json,
            ) {
                Ok(db) => db,
                Err(_) => PickleDb::new(
                    store_path,
                    PickleDbDumpPolicy::AutoDump,
                    SerializationMethod::Json,
                ),
            };

            Store {
                line_differences: VecDeque::new(),
                db,
            }
        }

        pub fn revert(&mut self) -> Result<bool, error::Error> {
            let line_difference = self.line_differences.pop_back().unwrap();
            self.db.rem(self.id(&line_difference).as_str())
        }

        pub fn store(&mut self, line_difference: &LineDifference) -> Result<(), error::Error> {
            self.line_differences.push_back(line_difference.clone());
            self.db.set(
                self.id(line_difference).as_str(),
                line_difference,
            )
        }

        fn id(&self, line_difference: &LineDifference) -> String {
            format!("{},{}", line_difference.path, line_difference.line_number)
        }
    }
}
