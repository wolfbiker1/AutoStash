use crate::util::{StatefulList, TabsState};

const TASKS: [&str; 2] = [
    "foo.txt", "bar.dat",
];

pub struct LineDifference<'a> {
    pub name: &'a str,
    pub location: &'a str
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub show_chart: bool,
    // pub progress: f64,
    pub tasks: StatefulList<&'a str>,
    pub servers: Vec<LineDifference<'a>>
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> App<'a> {
        App {
            title,
            should_quit: false,
            tabs: TabsState::new(vec![ "Statistic", "Info", "Overview"]),
            show_chart: true,
            // progress: 0.0,
            tasks: StatefulList::with_items(TASKS.to_vec()),

            servers: vec![
                LineDifference {
                    name: "foo",
                    location: "bar",
                },
            ],
        }
    }

    pub fn on_up(&mut self) {
        self.tasks.previous();
    }

    pub fn on_down(&mut self) {
        self.tasks.next();
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
    }
}
