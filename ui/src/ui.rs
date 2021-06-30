use crate::util::process_new_version;
use crate::util::{StatefulList, TabsState};
use crate::Event;
use crossterm::event::KeyEvent;
use diff::LineDifference;
use flume::{Receiver, Sender};
use store::store::{FileVersions, TimeFrame, HitsOfCode};
use tui::text::Spans;

static GRAPH_X_WIDTH: usize = 100;
///
/// contains channel pairs (tx & rx) which 
/// are used for backend - frontend communication
/// 
pub struct UICommunication {
    pub on_file_versions: Receiver<Vec<Option<FileVersions>>>,
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


///
/// contains a vector for selectable time slots
/// 
pub struct UITimeSlots {
    pub slots: Vec<TimeFrame>,
}


///
/// defines the app's title
/// 
pub struct UIConfig {
    pub title: String,
}


///
/// contains datastructures to store 
/// tui states like selected pane, visible versions,
/// snapshots, ...
/// 
pub struct UIState {
    pub file_versions: Vec<Option<FileVersions>>,
    pub filenames: StatefulList<String>,
    pub snapshots: StatefulList<String>,
    pub available_versions: Vec<String>,
    pub hits_of_codes_data: Vec<(f64, f64)>,
    pub tabs: TabsState,
    pub pane_ptr: i8,
    pub id_of_selected_file: usize,
    pub new_version: Vec<LineDifference>,
    pub processed_diffs: Vec<Spans<'static>>,
    pub path_of_selected_file: String,
    pub should_quit: bool,
}

impl UIState {




    pub fn update_hits_of_code(&mut self, id: usize/* , hits_of_code: Vec<HitsOfCode> */) {
        let h = &self.file_versions[id].clone().unwrap();
        let mut v: Vec<(f64, f64)> = Vec::new();
        v.push((0.0, 50.03));
        self.hits_of_codes_data = v;
        // let number_of_measurements = hits_of_code.len();

        // self.hits_of_codes_data.push((0.0, 50.03));
    }

    ///
    /// loads metainfo for the selected file and places all snapshots into the
    /// snapshot pane
    /// 
    pub fn update_file_pane(&mut self) {
        self.snapshots.flush_display();
        if let Some(i) = self.filenames.get_index() {
                self.id_of_selected_file = i;
        }
        let versions_for_selected_file = &self.file_versions[self.id_of_selected_file as usize];
        
        
        if versions_for_selected_file.is_none() {
            return;
        }
        let versions_for_selected_file = versions_for_selected_file.as_ref().unwrap();


        // temporary solution: better shift into own function
        let hits_of_code = versions_for_selected_file.hits_of_codes.clone();
        self.hits_of_codes_data.clear();
        if hits_of_code.len() < GRAPH_X_WIDTH {
            for (x, y) in hits_of_code.iter().enumerate() {
                self.hits_of_codes_data.push((x as f64, y.hits as f64));
            }
        } else {
            let scale_factor: f64 = GRAPH_X_WIDTH as f64 / hits_of_code.len() as f64;
            let mut x: f64 = 0.0;
            for y in hits_of_code.iter() {
                self.hits_of_codes_data.push((x, y.hits as f64));
                x += scale_factor;
            }   
        }
        // let mut v: Vec<(f64, f64)> = Vec::new();
        // v.push((0.0, 50.03));
        // self.hits_of_codes_data.push((0.0, 50.03));
        // update hits-of-code graph each time a file is selected
        // self.update_hits_of_code(self.id_of_selected_file as usize);

        self.path_of_selected_file = versions_for_selected_file.path.clone();
        for v in &versions_for_selected_file.versions {
            self.snapshots.add_item(v.datetime.to_string());
        }
    }
    ///
    /// loads the changes for the selected file, places them into diffpane
    /// 
    pub fn update_snapshot_pane(&mut self) {
        let selected_file = &self.file_versions[self.id_of_selected_file];
        if selected_file.is_none() {
            return;
        }
        let selected_file = selected_file.as_ref().unwrap();
        if !selected_file.versions.is_empty() {
            if let Some(i) = self.snapshots.get_index() {
                    let selected_version = &selected_file.versions[i];
                    let diffs_for_this_version = &selected_version.changes;
                    self.processed_diffs.clear();
                    self.processed_diffs = process_new_version(diffs_for_this_version.clone());
            }
        }
    }

    /// Reads state of selected pane
    /// 0 -> Pane for stored files
    /// 1 -> Pane for available snapshot for selected file
    pub fn update_pane_content(&mut self) {
        if self.pane_ptr > 0 {
            self.update_file_pane();
        } else {
            self.update_snapshot_pane();
        }
    }
    ///
    /// selects the following item in the list
    /// 
    pub fn on_up(&mut self) {
        if self.pane_ptr > 0 {
            self.filenames.previous();
        } else if !self.snapshots.list_is_empty() {
                self.snapshots.previous();
        }
        self.update_pane_content();
    }
    ///
    /// selects the following item in the list
    /// 
    pub fn on_down(&mut self) {
        if self.pane_ptr > 0 {
            self.filenames.next();
        } else if !self.snapshots.list_is_empty() {
                self.snapshots.next();
        }
        self.update_pane_content();
    }

    ///
    /// called on pressing arrow right. 
    /// increases the timeslice by 1 unit.
    /// this function clears the array where the difflines are stored,
    /// deselects any snapshot of the list, loads a new value into the
    /// snapshot buffer and the refreshs the difference pane.
    /// 
    pub fn on_right(&mut self) {
        self.processed_diffs.clear();
        self.snapshots.unselect();
        self.snapshots.flush_display();
        self.update_snapshot_pane();
        self.tabs.next();
        self.update_pane_content();
    }

    ///
    /// called on pressing arrow left. 
    /// decreases the timeslice by 1 unit.
    /// this function clears the array where the difflines are stored,
    /// deselects any snapshot of the list, loads a new value into the
    /// snapshot buffer and the refreshs the difference pane.
    /// 
    pub fn on_left(&mut self) {
        self.processed_diffs.clear();
        self.snapshots.unselect();
        self.snapshots.flush_display();
        self.update_snapshot_pane();
        self.tabs.previous();
        self.update_pane_content();
    }

    /// expects a char to process actions
    /// ```
    /// //will quit the application
    /// on_key('q') {}
    /// ```
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
}


///
/// contains subdatastructures for the UI
/// 
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
                hits_of_codes_data: Vec::new(),
                available_versions: Vec::new(),
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
