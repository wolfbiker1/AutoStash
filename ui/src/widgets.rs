use crate::ui::UI;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Dataset, List, ListItem, Paragraph, Tabs, Wrap,
    },
    Frame,
};

static IS_HIGHLIGHTED: Color = Color::Rgb(235, 203, 139);
static IS_BORDER: Color = Color::Rgb(129, 161, 193);
static IS_HEADLINE: Color = Color::Rgb(136, 192, 208);
static IS_LIGHT_WITE: Color = Color::Rgb(216, 222, 233);
static _IS_BACKGROUND_TEXT: Color = Color::Rgb(76, 86, 106);
static IS_WARNING: Color = Color::Rgb(208, 135, 112);

impl UI {
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());
        let titles = self
            .state
            .tabs
            .titles
            .iter()
            .map(|t| {
                Spans::from(Span::styled(
                    t.as_str(),
                    Style::default().fg(Color::Rgb(94, 129, 172)),
                ))
            })
            .collect();
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .border_style(Style::default().fg(IS_BORDER))
                    .borders(Borders::ALL)
                    .title("Timeslice"),
            )
            .highlight_style(Style::default().fg(IS_HIGHLIGHTED))
            .select(self.state.tabs.index);
        f.render_widget(tabs, chunks[0]);
        self.draw_tab(f, chunks[1])
    }

    fn draw_tab<B>(&mut self, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(30),
                    Constraint::Min(8),
                    Constraint::Length(7),
                    Constraint::Percentage(1),
                ]
                .as_ref(),
            )
            .split(area);
        self.draw_difference_pane(f, chunks[0]);
        self.draw_snapshot_pane(f, chunks[1]);
        self.draw_legend_pane(f, chunks[2]);
    }

    fn draw_difference_pane<B>(&self, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        Layout::default()
            .constraints(
                [
                    Constraint::Length(2),
                    // Constraint::Length(3),
                    // Constraint::Length(1),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(area);
        let block = Block::default()
            .border_style(Style::default().fg(IS_BORDER))
            .borders(Borders::ALL)
            .title("Differences");
        // let block = block.border_type(BorderType::Thick);
        let text: Vec<Spans> = self.state.processed_diffs.clone();
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    fn draw_legend_pane<B>(&self, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        Layout::default()
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(area);

        let undo_redo = Spans::from(vec![
            Span::styled(
                "esc ",
                Style::default().add_modifier(Modifier::BOLD).fg(IS_WARNING),
            ),
            Span::styled(
                "Undo",
                Style::default()
                    .add_modifier(Modifier::DIM)
                    .fg(IS_LIGHT_WITE),
            ),
            Span::from(" , "),
            Span::styled(
                "tab ",
                Style::default().add_modifier(Modifier::BOLD).fg(IS_WARNING),
            ),
            Span::styled(
                "Redo",
                Style::default()
                    .add_modifier(Modifier::DIM)
                    .fg(IS_LIGHT_WITE),
            ),
        ]);
        let modifier = Spans::from(vec![
            Span::styled(
                "s ",
                Style::default().add_modifier(Modifier::BOLD).fg(IS_WARNING),
            ),
            Span::styled(
                "switch panes",
                Style::default()
                    .add_modifier(Modifier::DIM)
                    .fg(IS_LIGHT_WITE),
            ),
        ]);
        let arrow_up_down = Spans::from(vec![
            Span::styled(
                "▲ ",
                Style::default().add_modifier(Modifier::BOLD).fg(IS_WARNING),
            ),
            Span::styled(
                "Line above",
                Style::default()
                    .add_modifier(Modifier::DIM)
                    .fg(IS_LIGHT_WITE),
            ),
            Span::from(" , "),
            Span::styled(
                "▼ ",
                Style::default().add_modifier(Modifier::BOLD).fg(IS_WARNING),
            ),
            Span::styled(
                "Line below",
                Style::default()
                    .add_modifier(Modifier::DIM)
                    .fg(IS_LIGHT_WITE),
            ),
        ]);
        let arrow_left_right = Spans::from(vec![
            Span::styled(
                "◄ ",
                Style::default().add_modifier(Modifier::BOLD).fg(IS_WARNING),
            ),
            Span::styled(
                "Decrease Timeslice",
                Style::default()
                    .add_modifier(Modifier::DIM)
                    .fg(IS_LIGHT_WITE),
            ),
            Span::from(" , "),
            Span::styled(
                "► ",
                Style::default().add_modifier(Modifier::BOLD).fg(IS_WARNING),
            ),
            Span::styled(
                "Increase Timeslice",
                Style::default()
                    .add_modifier(Modifier::DIM)
                    .fg(IS_LIGHT_WITE),
            ),
        ]);
        let quit = Spans::from(vec![
            Span::styled(
                "q ",
                Style::default().add_modifier(Modifier::BOLD).fg(IS_WARNING),
            ),
            Span::styled(
                "Quit",
                Style::default()
                    .add_modifier(Modifier::DIM)
                    .fg(IS_LIGHT_WITE),
            ),
        ]);
        let block = Block::default()
            .border_style(Style::default().fg(IS_BORDER))
            .borders(Borders::ALL)
            .title("Shortcuts");
        let text: Vec<Spans> = vec![quit, modifier, undo_redo, arrow_left_right, arrow_up_down];
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    // pane 1
    fn draw_snapshot_pane<B>(&mut self, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let constrains = vec![Constraint::Percentage(50), Constraint::Percentage(50)];
        let chunks = Layout::default()
            .constraints(constrains)
            .direction(Direction::Horizontal)
            .split(area);
        {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[0]);
            {
                let chunks = Layout::default()
                    .constraints([Constraint::Percentage(100), Constraint::Percentage(50)].as_ref())
                    .direction(Direction::Horizontal)
                    .split(chunks[0]);

                let snapshots: Vec<ListItem> = self
                    .state
                    .snapshots
                    .items
                    .iter()
                    .map(|i| ListItem::new(vec![Spans::from(Span::raw(i.as_str()))]))
                    .collect();
                let mut snapshots = List::new(snapshots)
                    .style(Style::default().fg(IS_LIGHT_WITE))
                    .highlight_symbol("►")
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(IS_HIGHLIGHTED),
                    );
                if self.state.pane_ptr == 1 {
                    snapshots = snapshots.block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(IS_BORDER))
                            .title(Span::styled(
                                "Available Snapshot",
                                Style::default().fg(IS_HEADLINE),
                            )),
                    );
                } else {
                    snapshots = snapshots.block(
                        Block::default()
                            .title(Span::styled(
                                "Available Snapshot",
                                Style::default().fg(IS_HEADLINE),
                            ))
                            .border_style(Style::default().fg(IS_BORDER))
                            .borders(Borders::ALL)
                            .border_type(BorderType::Thick),
                    );
                }

                f.render_stateful_widget(snapshots, chunks[0], &mut self.state.snapshots.state);
            }

            {
                Layout::default()
                    .constraints([Constraint::Percentage(100), Constraint::Percentage(50)].as_ref())
                    .direction(Direction::Horizontal)
                    .split(chunks[0]);
            }
            let filenames_list: Vec<ListItem> = self
                .state
                .filenames
                .items
                .iter()
                .map(|i| ListItem::new(vec![Spans::from(Span::raw(i.as_str()))]))
                .collect();

            let mut filenames = List::new(filenames_list)
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(IS_HIGHLIGHTED),
                )
                .highlight_symbol("►")
                .style(Style::default().fg(IS_LIGHT_WITE));

            if self.state.pane_ptr == 1 {
                filenames = filenames.block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(IS_BORDER))
                        .border_type(BorderType::Thick)
                        .title(Span::styled("Filename", Style::default().fg(IS_HEADLINE))),
                );
            } else {
                filenames = filenames.block(
                    Block::default()
                        .border_style(Style::default().fg(IS_BORDER))
                        .borders(Borders::ALL)
                        .title(Span::styled(
                            "Filename",
                            Style::default().fg(Color::Rgb(94, 129, 172)),
                        )),
                );
            }
            f.render_stateful_widget(filenames, chunks[1], &mut self.state.filenames.state);
        }
        let x_labels = vec![];
        let datasets = vec![Dataset::default()
            .name("Hits of Code")
            .marker(symbols::Marker::Braille)
            .style(
                Style::default()
                    .fg(IS_HIGHLIGHTED)
                    .add_modifier(Modifier::DIM),
            )
            .data(&[
                (0.0, 0.0),
                (1.0, 1.0),
                (2.0, 2.0),
                (3.0, 3.0),
                (4.0, 4.0),
                (5.0, 5.0),
                (6.0, 6.0),
                (7.0, 7.0),
                (8.0, 8.0),
                (9.0, 9.0),
                (10.0, 10.0),
                (11.0, 11.0),
                (12.0, 12.0),
                (13.0, 13.0),
                (14.0, 14.0),
                (15.0, 15.0),
                (16.0, 16.0),
                (17.0, 17.0),
                (18.0, 18.0),
                (19.0, 19.0),
                (20.0, 20.0),
                (21.0, 21.0),
                (22.0, 22.0),
                (23.0, 23.0),
                (24.0, 24.0),
                (25.0, 25.0),
                (26.0, 26.0),
                (27.0, 27.0),
                (28.0, 28.0),
                (29.0, 29.0),
                (30.0, 30.0),
                (31.0, 31.0),
                (32.0, 32.0),
                (33.0, 33.0),
                (34.0, 34.0),
                (35.0, 35.0),
                (36.0, 36.0),
                (37.0, 37.0),
                (38.0, 38.0),
                (39.0, 39.0),
                (40.0, 40.0),
                (41.0, 41.0),
                (42.0, 42.0),
                (43.0, 43.0),
                (44.0, 44.0),
                (45.0, 45.0),
                (46.0, 46.0),
                (47.0, 47.0),
                (48.0, 48.0),
                (49.0, 49.0),
                (50.0, 50.0),
                (51.0, 51.0),
                (52.0, 52.0),
                (53.0, 53.0),
                (54.0, 54.0),
                (55.0, 55.0),
                (56.0, 56.0),
                (57.0, 57.0),
                (58.0, 58.0),
                (59.0, 59.0),
                (60.0, 60.0),
                (61.0, 61.0),
                (62.0, 62.0),
                (63.0, 63.0),
                (64.0, 64.0),
                (65.0, 65.0),
                (66.0, 66.0),
                (67.0, 67.0),
                (68.0, 68.0),
                (69.0, 69.0),
                (70.0, 70.0),
                (71.0, 71.0),
                (72.0, 72.0),
                (73.0, 73.0),
                (74.0, 74.0),
                (75.0, 75.0),
                (76.0, 76.0),
                (77.0, 77.0),
                (78.0, 78.0),
                (79.0, 79.0),
                (80.0, 80.0),
                (81.0, 81.0),
                (82.0, 82.0),
                (83.0, 83.0),
                (84.0, 84.0),
                (85.0, 85.0),
                (86.0, 86.0),
                (87.0, 87.0),
                (88.0, 88.0),
                (89.0, 89.0),
                (90.0, 90.0),
                (91.0, 91.0),
                (92.0, 92.0),
                (93.0, 93.0),
                (94.0, 94.0),
                (95.0, 95.0),
                (96.0, 96.0),
                (97.0, 97.0),
                (98.0, 98.0),
                (99.0, 99.0),
            ])];
        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Hits-Of-Code",
                        Style::default().fg(IS_HEADLINE),
                    ))
                    .border_style(Style::default().fg(IS_BORDER))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("Date")
                    .style(
                        Style::default()
                            .add_modifier(Modifier::DIM)
                            .fg(IS_LIGHT_WITE),
                    )
                    .bounds([0.0, 100.0])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    // .title("Hits-of-code * 10000")
                    .style(
                        Style::default()
                            .add_modifier(Modifier::DIM)
                            .fg(IS_LIGHT_WITE),
                    )
                    .bounds([0.0, 100.0])
                    .labels(vec![
                        Span::styled(
                            "0",
                            Style::default()
                                .fg(IS_LIGHT_WITE)
                                .add_modifier(Modifier::DIM),
                        ),
                        Span::styled(
                            "100",
                            Style::default()
                                .fg(IS_LIGHT_WITE)
                                .add_modifier(Modifier::DIM),
                        ),
                    ]),
            );
        f.render_widget(chart, chunks[1]);
    }
}
