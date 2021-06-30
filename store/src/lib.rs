extern crate simple_error;

pub mod store {
    use chrono::NaiveDate;
    use chrono::NaiveDateTime;
    use chrono::Utc;
    use diff::LineDifference;
    use itertools::Itertools;
    use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::error;
    use std::fs::File;
    use std::io::Write;
    use std::io::{self, BufRead};
    use walkdir::{DirEntry, WalkDir};

    static FILE_VERSION_STACK: &str = "FILE_VERSION_STACK";
    static FILE_VERSION_MARKER: &str = "FILE_VERSION_MARKER";
    pub struct Store {
        db: PickleDb,
        pub time_frame: TimeFrame,
    }

    #[derive(Clone, Debug)]
    pub struct Version {
        pub datetime: NaiveDateTime,
        pub changes: Vec<LineDifference>,
    }

    #[derive(Clone, Debug)]
    pub struct FileVersions {
        pub path: String,
        pub versions: Vec<Version>,
        pub hits_of_codes: Vec<HitsOfCode>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    struct VersionStack {
        path: String,
        timestamps: Vec<i64>,
    }

    #[derive(Serialize, Deserialize)]
    struct VersionMarker {
        path: String,
        // Test
        timestamp_marker: usize,
    }

    #[derive(Clone)]
    pub enum TimeFrame {
        MINUTE,
        HOUR,
        DAY,
        WEEK,
    }

    #[derive(Clone, Debug)]
    pub struct HitsOfCode {
        pub date: NaiveDate,
        pub hits: u64,
    }

    impl TimeFrame {
        pub fn value(&self) -> i64 {
            match &self {
                &Self::MINUTE => 60,
                &Self::HOUR => 60 * &Self::MINUTE.value(),
                &Self::DAY => 24 * &Self::HOUR.value(),
                &Self::WEEK => 7 * &Self::DAY.value(),
            }
        }
    }

    fn init(
        store_path: &str,
        watch_path: &str,
        excluded_files: Vec<String>,
        excluded_paths: Vec<String>,
    ) -> Result<PickleDb, Box<dyn error::Error>> {
        let mut db = PickleDb::new(
            store_path,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Yaml,
        );

        create_stack(&mut db)?;
        init_store(watch_path, &mut db, excluded_files, excluded_paths)?;

        Ok(db)
    }

    fn create_stack(db: &mut PickleDb) -> Result<(), pickledb::error::Error> {
        db.lcreate(FILE_VERSION_STACK)?;
        db.lcreate(FILE_VERSION_MARKER).map(|_| ())
    }

    fn hits_of_codes(file_versions: Vec<Version>) -> Vec<HitsOfCode> {
        file_versions
            .iter()
            .map(|version| HitsOfCode {
                date: version.datetime.date(),
                hits: version.changes.iter().count() as u64,
            })
            .collect_vec()
            .into_iter()
            .coalesce(|prev, next| {
                if prev.date.eq(&next.date) {
                    return Ok(HitsOfCode {
                        date: prev.date,
                        hits: prev.hits + next.hits,
                    });
                }
                Err((prev, next))
            })
            .collect_vec()
    }

    fn init_store(
        watch_path: &str,
        db: &mut PickleDb,
        excluded_files: Vec<String>,
        excluded_paths: Vec<String>,
    ) -> Result<(), Box<dyn error::Error>> {
        WalkDir::new(watch_path)
            .into_iter()
            .filter(|entry| match entry {
                Ok(entry) => is_not_excluded(entry, &excluded_files, &excluded_paths),
                _ => false,
            })
            .map(
                |entry: Result<DirEntry, walkdir::Error>| -> Result<(), Box<dyn error::Error>> {
                    let entry = entry?;
                    let path = entry.path();
                    let path = path.to_str().unwrap_or_else(|| "couldn't find path");

                    init_file_version_stack(path.to_string(), db);
                    init_file_version_marker(path.to_string(), db);
                    init_file_changes(path.to_string(), db)
                },
            )
            .find(|e| e.is_err())
            .unwrap_or_else(|| Ok(()))
    }

    fn is_not_excluded(
        entry: &DirEntry,
        excluded_files: &Vec<String>,
        excluded_paths: &Vec<String>,
    ) -> bool {
        entry.path().is_file()
            && !excluded_files.contains(
                &entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
            && excluded_paths
                .iter()
                .all(|p| !entry.path().parent().unwrap().ends_with(p))
    }

    fn init_file_version_stack(path: String, db: &mut PickleDb) {
        let timestamps = vec![Utc::now().naive_utc().timestamp()];
        let version_stack = VersionStack { path, timestamps };

        db.ladd(FILE_VERSION_STACK, &version_stack);
    }

    fn init_file_version_marker(path: String, db: &mut PickleDb) {
        let version_marker = VersionMarker {
            path,
            timestamp_marker: 1,
        };

        db.ladd(FILE_VERSION_MARKER, &version_marker);
    }

    fn init_file_changes(path: String, db: &mut PickleDb) -> Result<(), Box<dyn error::Error>> {
        db.lcreate(path.as_str())?;
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
        pub fn new(
            store_path: &str,
            watch_path: &str,
            excluded_files: Vec<String>,
            excluded_paths: Vec<String>,
        ) -> Result<Store, Box<dyn error::Error>> {
            let mut db = load(store_path);
            if db.is_err() {
                db = init(store_path, watch_path, excluded_files, excluded_paths);
            }
            let db = db?;

            Ok(Store {
                db,
                time_frame: TimeFrame::HOUR,
            })
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
            let version_stack = self.db.liter(FILE_VERSION_STACK).find(|version_stack| {
                let version_stack: VersionStack = version_stack.get_item().unwrap();
                version_stack.path.eq(path)
            });
            if version_stack.is_some() {
                let now = Utc::now().naive_utc().timestamp();
                let mut version_stack: VersionStack = version_stack.unwrap().get_item().unwrap();
                let version_marker: VersionMarker =
                    self.get_version_marker(path.to_string()).unwrap();

                self.db.lrem_value(FILE_VERSION_STACK, &version_stack)?;
                version_stack.timestamps.push(now);
                self.db.ladd(FILE_VERSION_STACK, &version_stack);

                self.increment_version_marker_by(version_marker, 1);
            } else {
                init_file_version_stack(path.to_string(), &mut self.db);
                init_file_version_marker(path.to_string(), &mut self.db);
            }

            Ok(())
        }

        pub fn change_time_frame(&mut self, time_frame: TimeFrame) {
            self.time_frame = time_frame;
        }

        pub fn undo_by(&mut self, path: String, count: usize) -> Result<(), Box<dyn error::Error>> {
            self.undo(path, count)
        }

        pub fn redo_by(&mut self, path: String, count: usize) -> Result<(), Box<dyn error::Error>> {
            self.redo(path, count)
        }

        pub fn view(&mut self) -> Result<Vec<Option<FileVersions>>, Box<dyn error::Error>> {
            let now = Utc::now().naive_utc();

            Ok(self
                .db
                .liter(FILE_VERSION_STACK)
                .map(|version_stack| {
                    let version_stack: VersionStack = version_stack.get_item().unwrap();
                    let timestamps = version_stack
                        .timestamps
                        .iter()
                        .filter(|timestamp| {
                            now.timestamp() - self.time_frame.value() < *timestamp.clone()
                        })
                        .map(|timestamp| timestamp.clone())
                        .collect_vec();

                    if timestamps.is_empty() {
                        return None;
                    }
                    Some(VersionStack {
                        timestamps,
                        path: version_stack.path,
                    })
                })
                .map(|version_stack| -> Option<FileVersions> {
                    if version_stack.is_none() {
                        return None;
                    }
                    let mut version_stack = version_stack.unwrap();
                    let path = version_stack.clone().path;
                    version_stack.timestamps.push(0);
                    let versions = self.get_versions(version_stack);
                    let hits_of_codes = hits_of_codes(versions.clone());

                    Some(FileVersions {
                        path,
                        versions,
                        hits_of_codes,
                    })
                })
                .collect_vec())
        }

        fn get_versions(&self, version_stack: VersionStack) -> Vec<Version> {
            if version_stack.timestamps.len() == 1 {
                return version_stack
                    .timestamps
                    .iter()
                    .map(|timestamp| {
                        let changes: Vec<LineDifference> = self
                            .get_file_changes::<LineDifference>(version_stack.path.as_str())
                            .iter()
                            .sorted_by(|a, b| {
                                diff::sort(b.date_time.as_str(), a.date_time.as_str())
                            })
                            .filter(|e| {
                                let diff_timestamp = NaiveDateTime::parse_from_str(
                                    e.date_time.as_str(),
                                    diff::RFC3339,
                                )
                                .unwrap()
                                .timestamp();

                                diff_timestamp >= timestamp.clone()
                            })
                            .map(|e| e.clone())
                            .collect_vec();

                        Version {
                            datetime: NaiveDateTime::from_timestamp(timestamp.clone(), 0),
                            changes,
                        }
                    })
                    .collect_vec();
            }

            version_stack
                .timestamps
                .iter()
                .sorted_by(|a, b| Ord::cmp(b, a))
                .collect_vec()
                .windows(2)
                .map(|diff_pairs| {
                    let changes = self
                        .get_file_changes::<LineDifference>(version_stack.path.as_str())
                        .iter()
                        .sorted_by(|a, b| diff::sort(b.date_time.as_str(), a.date_time.as_str()))
                        .filter(|e| {
                            let diff_timestamp =
                                NaiveDateTime::parse_from_str(e.date_time.as_str(), diff::RFC3339)
                                    .unwrap()
                                    .timestamp();

                            diff_timestamp <= diff_pairs[0].clone()
                                && diff_timestamp > *diff_pairs[1]
                        })
                        .map(|e| e.clone())
                        .collect_vec();

                    Version {
                        datetime: NaiveDateTime::from_timestamp(diff_pairs[0].clone(), 0),
                        changes,
                    }
                })
                .collect_vec()
        }

        fn get_version_marker(&self, path: String) -> Option<VersionMarker> {
            self.db
                .liter(FILE_VERSION_MARKER)
                .find(|version_marker| {
                    let version_marker: VersionMarker = version_marker.get_item().unwrap();
                    version_marker.path.eq(&path)
                })
                .map(|version_marker| version_marker.get_item().unwrap())
        }

        fn undo(&mut self, path: String, count: usize) -> Result<(), Box<dyn error::Error>> {
            let versions = self.peek_versions(path.clone(), count, true);
            let version_marker: VersionMarker = self.get_version_marker(path.to_string()).unwrap();

            self.decrement_version_marker_by(version_marker, count);
            self.undo_versions(versions)
        }

        fn decrement_version_marker_by(&mut self, mut version_marker: VersionMarker, count: usize) {
            self.db
                .lrem_value(FILE_VERSION_MARKER, &version_marker)
                .unwrap();
            version_marker.timestamp_marker -= count;
            self.db.ladd(FILE_VERSION_MARKER, &version_marker);
        }

        fn increment_version_marker_by(&mut self, mut version_marker: VersionMarker, count: usize) {
            self.db
                .lrem_value(FILE_VERSION_MARKER, &version_marker)
                .unwrap();
            version_marker.timestamp_marker += count;
            self.db.ladd(FILE_VERSION_MARKER, &version_marker);
        }

        fn redo(&mut self, path: String, count: usize) -> Result<(), Box<dyn error::Error>> {
            let versions = self.peek_versions(path.clone(), count, true);
            let version_marker: VersionMarker = self.get_version_marker(path.to_string()).unwrap();

            self.increment_version_marker_by(version_marker, count);
            self.undo_versions(versions)
        }

        pub fn get_file_changes<T: DeserializeOwned + std::fmt::Debug>(
            &self,
            path: &str,
        ) -> Vec<T> {
            self.db
                .liter(path)
                .map(|e| e.get_item::<T>().unwrap())
                .collect()
        }

        fn peek_versions(&mut self, path: String, count: usize, previous: bool) -> Vec<Version> {
            let version_marker = self.get_version_marker(path.clone()).unwrap();
            let pos;
            if previous {
                pos = version_marker.timestamp_marker - count;
            } else {
                pos = version_marker.timestamp_marker;
            }

            let mut version_stack = self
                .db
                .liter(FILE_VERSION_STACK)
                .map(|version_stack| version_stack.get_item().unwrap())
                .find(|version_stack: &VersionStack| version_stack.path.eq(&path))
                .unwrap();

            version_stack.timestamps = version_stack
                .timestamps
                .iter()
                .skip(pos)
                .take(count)
                .map(|timestamp| timestamp.clone())
                .collect_vec();

            self.get_versions(
                self.db
                    .liter(FILE_VERSION_STACK)
                    .map(|version_stack| version_stack.get_item().unwrap())
                    .find(|version_stack: &VersionStack| version_stack.path.eq(&path))
                    .unwrap(),
            )
        }

        fn undo_versions(&self, versions: Vec<Version>) -> Result<(), Box<dyn error::Error>> {
            versions.iter().for_each(|version| {
                // TODO: propagate error
                self.undo_changes(&version.changes).unwrap()
            });
            Ok(())
        }

        fn undo_changes(&self, changes: &Vec<LineDifference>) -> Result<(), Box<dyn error::Error>> {
            let path = changes.first().unwrap().path.clone();
            let file = File::open(path.clone())?;
            let lines = io::BufReader::new(file).lines().collect_vec();
            let mut redone_lines: Vec<String> = vec![];

            if lines.len() == 0 {
                redone_lines.push(
                    changes
                        .iter()
                        .find(|l| l.line_number.eq(&0))
                        .unwrap()
                        .line
                        .clone(),
                );
            } else {
                redone_lines = lines
                    .iter()
                    .map(|l| l.as_ref().unwrap())
                    .enumerate()
                    .map(|(index, line)| {
                        let found = changes.iter().find(|l| l.line_number.eq(&index));
                        if found.is_some() {
                            let found = found.unwrap();
                            return found.line.clone();
                        }
                        line.clone()
                    })
                    .collect();
            }

            let mut file = File::create(path.clone())?;
            file.write_all(redone_lines.join("\n").as_bytes())
                .map_err(|err| err.into())
        }
    }
}
