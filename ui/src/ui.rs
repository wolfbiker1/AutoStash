use crate::util::{StatefulList, TabsState};
use crate::Event;
use crossterm::event::KeyEvent;
use diff::LineDifference;
use flume::{Receiver, Sender};
use store::store::Version;
use tui::text::Spans;

pub struct UICommunication {
    pub on_versions: Receiver<Vec<Version>>,
    pub on_key: Receiver<Event<KeyEvent>>,
    pub on_quit: Receiver<()>,
    pub undo_to_handle: Sender<usize>,
    pub redo_to_handle: Sender<usize>,
    pub key_to_ui: Sender<Event<KeyEvent>>,
    pub quit_to_ui: Sender<()>,
    pub quit_to_handle: Sender<()>,
}

impl UICommunication {
    pub fn on_undo(&mut self) {
        self.undo_to_handle.send(1).unwrap_or_else(|err| {
            eprintln!("Could not undo step: {:?}", err);
        });
    }
    pub fn on_redo(&mut self) {
        self.redo_to_handle.send(1).unwrap_or_else(|err| {
            eprintln!("Could not redo step: {:?}", err);
        });
    }
}

pub struct UIConfig {
    pub title: String,
    pub show_chart: bool,
}

pub struct UIState {
    pub all_versions: Vec<Version>,
    pub filenames: StatefulList<String>,
    pub lines: StatefulList<String>,
    pub available_versions: Vec<String>,
    pub tabs: TabsState,
    pub pane_ptr: i8,
    pub new_version: Vec<LineDifference>,
    pub processed_diffs: Vec<Spans<'static>>,
    pub should_quit: bool,
}

impl UIState {
    pub fn on_up(&mut self) {
        if self.pane_ptr > 0 {
            self.filenames.previous();
        } else {
            self.lines.previous();
        }
    }

    pub fn on_enter(&mut self) {
        if self.pane_ptr > 0 {
            self.lines.flush_display();
            let data_for_selected_file = &self.all_versions[self.filenames.get_index()];
            let diffs = data_for_selected_file.changes.clone();
            // let diffs = r.changes.clone();
            for d in diffs {
                self.lines.add_item(d.line);
            }
            // println!("{}", self.filenames.get_index());
        } else {
        }
    }

    pub fn on_down(&mut self) {
        if self.pane_ptr > 0 {
            self.filenames.next();
        } else {
            self.lines.next();
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
        // todo: change timeslice
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
        // todo: change timeslice
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
                all_versions: Vec::new(),
                lines: StatefulList::with_items(vec![]),
                filenames: StatefulList::with_items(vec![]),
                available_versions: Vec::new(),
                processed_diffs: Vec::new(),
                new_version: Vec::new(),
                pane_ptr: 1,
            },
            communication,
        }
    }
}
