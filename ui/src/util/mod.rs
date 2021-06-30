use diff::LineDifference;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::ListState;
pub struct TabsState {
    pub titles: Vec<String>,
    pub index: usize,
}

impl TabsState {
    pub fn new(titles: Vec<String>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub fn process_new_version(diffs: Vec<LineDifference>) -> Vec<Spans<'static>> {
    let mut v: Vec<Span> = vec![];
    let mut spans: Vec<Spans> = vec![];
    for diff in &diffs {
        v.push(Span::raw("\n"));
        v.push(Span::styled(
            diff.line_number.to_string(),
            Style::default().fg(Color::Blue),
        ));
        v.push(Span::raw("->"));
        v.push(Span::styled(
            diff.line.clone(),
            Style::default().fg(Color::Red),
        ));
        v.push(Span::raw("->"));
        v.push(Span::styled(
            diff.changed_line.clone(),
            Style::default().fg(Color::Green),
        ));
        v.push(Span::raw("\n"));
        spans.push(Spans::from(v.clone()));
        v.clear();
    }
    spans
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn flush_display(&mut self) {
        self.items.clear();
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn get_index(&mut self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
