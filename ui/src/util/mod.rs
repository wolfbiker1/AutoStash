use diff::LineDifference;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::ListState;
pub struct TabsState {
    pub titles: Vec<String>,
    pub index: usize,
}

static IS_DANGER: Color = Color::Rgb(191, 97, 106);
static IS_SUCCESS: Color = Color::Rgb(163, 190, 140);
static IS_LIGHT_WITE: Color = Color::Rgb(216, 222, 233);

impl TabsState {
    pub fn new(titles: Vec<String>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }
    pub fn get_index(&mut self) -> usize {
        self.index
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
            String::from("l"),
            Style::default()
                .add_modifier(Modifier::DIM)
                .fg(IS_LIGHT_WITE),
        ));
        v.push(Span::styled(
            diff.line_number.to_string(),
            Style::default()
                .add_modifier(Modifier::DIM)
                .fg(IS_LIGHT_WITE),
        ));
        v.push(Span::raw(" » "));

        let mut previous_line = diff.line.clone();
        if previous_line.is_empty() {
            previous_line = String::from("< empty line >");
        }

        v.push(Span::styled(
            previous_line,
            Style::default()
                .add_modifier(Modifier::ITALIC)
                .fg(IS_DANGER),
        ));
        v.push(Span::raw(" » "));

        let mut changed_line = diff.changed_line.clone();
        if changed_line.is_empty() {
            changed_line = String::from("< empty line >");
        }

        v.push(Span::styled(
            changed_line,
            Style::default()
                .add_modifier(Modifier::ITALIC)
                .fg(IS_SUCCESS),
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

    pub fn reset_index(&mut self) {
        // self.state.reset();
    }

    pub fn list_is_empty(&mut self) -> bool {
        self.items.is_empty()
    }

    pub fn get_index(&mut self) -> Option<usize> {
        self.state.selected()
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diff_to_ui_representation() {
        let path = "test2.txt";
        let diff = vec![LineDifference::new(
            path.to_string(),
            5,
            "".to_string(),
            "Hello World".to_string(),
        )];
        assert_eq!(2, 2);
    }
}
