use crate::Event;
use crate::util::{StatefulList, TabsState};
use crossterm::event::KeyEvent;
use diff::LineDifference;
use std::sync::mpsc;
use store::store::Version;
use tui::text::Spans;

pub struct UICommunication {
    pub on_lines: mpsc::Receiver<Vec<LineDifference>>,
    pub on_versions: mpsc::Receiver<Vec<Version>>,
    pub undo_to_handle: mpsc::Sender<usize>,
    pub redo_to_handle: mpsc::Sender<usize>,
    pub on_key:  mpsc::Receiver<Event<KeyEvent>>,
    pub key_to_ui:  mpsc::Sender<Event<KeyEvent>>,
}

pub struct UIConfig {
    pub title: String,
    pub should_quit: bool,
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
                should_quit: false,
                show_chart: true,
            },
            state: UIState {
                tabs: TabsState::new(vec!["1h".to_string(), "24h".to_string(), "7 Tage".to_string()]),
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
    pub fn on_up(&mut self) {
        if self.state.pane_ptr > 0 {
            self.state.filenames.previous();
        } else {
            self.state.lines.previous();
        }
    }

    pub fn on_enter(&mut self) {
        if self.state.pane_ptr > 0 {
            self.state.lines.flush_display();
            let data_for_selected_file = &self.state.all_versions[self.state.filenames.get_index()];
            let diffs = data_for_selected_file.changes.clone();
            // let diffs = r.changes.clone();
            for d in diffs {
                self.state.lines
                    .add_item(d.line);
            }
            // println!("{}", self.filenames.get_index());
        } else {
        }
    }

    pub fn on_down(&mut self) {
        if self.state.pane_ptr > 0 {
            self.state.filenames.next();
        } else {
            self.state.lines.next();
        }
    }

    pub fn on_right(&mut self) {
        self.state.tabs.next();
        // todo: change timeslice
    }

    pub fn on_left(&mut self) {
        self.state.tabs.previous();
        // todo: change timeslice
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.config.should_quit = true;
            }
            's' => {
                self.state.pane_ptr *= -1;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {}
}
