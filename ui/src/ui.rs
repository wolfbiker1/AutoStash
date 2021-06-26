use crate::util::process_new_version;
use crate::util::{StatefulList, TabsState};
use crate::Event;
use crossterm::event::KeyEvent;
use diff::LineDifference;
use flume::{Receiver, Sender};
use store::store::{FileVersions, TimeFrame};
use tui::text::Spans;

pub struct UICommunication {
    pub on_file_versions: Receiver<Vec<FileVersions>>,
    pub on_key: Receiver<Event<KeyEvent>>,
    pub on_quit: Receiver<()>,
    pub undo_to_handle: Sender<(String, usize)>,
    pub redo_to_handle: Sender<(String, usize)>,
    pub time_frame_change_to_handle: Sender<TimeFrame>,
    pub key_to_ui: Sender<Event<KeyEvent>>,
    pub quit_to_ui: Sender<()>,
    pub quit_to_handle: Sender<()>,
}

impl UICommunication {
    pub fn on_undo(&mut self, path: String, count: usize) {
        self.undo_to_handle
            .send((path, count))
            .unwrap_or_else(|err| {
                eprintln!("Could not undo step: {:?}", err);
            });
    }
    pub fn on_redo(&mut self, path: String, count: usize) {
        self.redo_to_handle
            .send((path, count))
            .unwrap_or_else(|err| {
                eprintln!("Could not redo step: {:?}", err);
            });
    }
    pub fn on_timeslice_change(&mut self, selected_slot: usize) {
        let time_frame: TimeFrame;
        match selected_slot {
            0 => time_frame = TimeFrame::HOUR,
            1 => time_frame = TimeFrame::DAY,
            2 => time_frame = TimeFrame::WEEK,
            // satifsy compiler
            _ => time_frame = TimeFrame::DAY,
        }
        self.time_frame_change_to_handle
            .send(time_frame)
            .unwrap_or_else(|err| {
                eprintln!("Could not set timeframe: {:?}", err);
            });
    }
}

pub struct UITimeSlots {
    pub slots: Vec<TimeFrame>,
}

impl UITimeSlots {
    pub fn new() /* -> self */ {}
}

pub struct UIConfig {
    pub title: String,
    pub show_chart: bool,
}

pub struct UIState {
    pub file_versions: Vec<FileVersions>,
    pub filenames: StatefulList<String>,
    pub all_versions: Vec<FileVersions>,
    pub snapshots: StatefulList<String>,
    pub available_versions: Vec<String>,
    pub tabs: TabsState,
    pub pane_ptr: i8,
    pub id_of_selected_file: usize,
    pub new_version: Vec<LineDifference>,
    pub processed_diffs: Vec<Spans<'static>>,
    pub path_of_selected_file: String,
    pub should_quit: bool,
}

impl UIState {
    pub fn update_file_pane(&mut self) {
        self.snapshots.flush_display();
        match self.filenames.get_index() {
            Some(i) => {
                self.id_of_selected_file = i;
            }
            None => {}
        }
        let versions_for_selected_file = &self.file_versions[self.id_of_selected_file as usize];
        self.path_of_selected_file = versions_for_selected_file.path.clone();
        for v in &versions_for_selected_file.versions {
            self.snapshots.add_item(v.datetime.to_string());
        }
    }

    pub fn update_snapshot_pane(&mut self) {
        let selected_file = &self.file_versions[self.id_of_selected_file];

        if &selected_file.versions.len() > &0 {
            match self.snapshots.get_index() {
                Some(i) => {
                    let selected_version = &selected_file.versions[i];
                    let diffs_for_this_version = &selected_version.changes;
                    self.processed_diffs.clear();
                    self.processed_diffs = process_new_version(diffs_for_this_version.clone());
                }
                None => {}
            }
        }
    }

    pub fn update_pane_content(&mut self) {
        if self.pane_ptr > 0 {
            self.update_file_pane();
        } else {
            self.update_snapshot_pane();
        }
    }

    pub fn on_up(&mut self) {
        if self.pane_ptr > 0 {
            self.filenames.previous();
        } else {
            if !self.snapshots.list_is_empty() {
                self.snapshots.previous();
            }
        }
        self.update_pane_content();
    }

    pub fn on_down(&mut self) {
        if self.pane_ptr > 0 {
            self.filenames.next();
        } else {
            if !self.snapshots.list_is_empty() {
                self.snapshots.next();
            }
        }
        self.update_pane_content();
    }

    pub fn on_right(&mut self) {
        self.processed_diffs.clear();
        self.snapshots.unselect();
        self.snapshots.flush_display();
        self.update_snapshot_pane();
        self.tabs.next();
        self.update_pane_content();
    }

    pub fn on_left(&mut self) {
        self.processed_diffs.clear();
        self.snapshots.unselect();
        self.snapshots.flush_display();
        self.update_snapshot_pane();
        self.tabs.previous();
        self.update_pane_content();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            's' => {
                self.pane_ptr *= -1;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {}
}

pub struct UI {
    pub config: UIConfig,
    pub state: UIState,
    pub communication: UICommunication,
    pub timeslots: UITimeSlots,
}

impl UI {
    pub fn new(title: String, communication: UICommunication) -> UI {
        UI {
            config: UIConfig {
                title,
                show_chart: true,
            },
            state: UIState {
                tabs: TabsState::new(vec![
                    "1h".to_string(),
                    "24h".to_string(),
                    "7 Tage".to_string(),
                ]),
                should_quit: false,
                file_versions: Vec::new(),
                snapshots: StatefulList::with_items(vec![]),
                filenames: StatefulList::with_items(vec![String::from("loading...")]),
                available_versions: Vec::new(),
                all_versions: Vec::new(),
                processed_diffs: Vec::new(),
                new_version: Vec::new(),
                path_of_selected_file: String::new(),
                id_of_selected_file: 0,
                pane_ptr: 1,
            },
            communication,
            timeslots: UITimeSlots {
                slots: vec![TimeFrame::HOUR, TimeFrame::DAY, TimeFrame::WEEK],
            },
        }
    }
}
