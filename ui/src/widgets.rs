pub mod widgets {
    use crate::ui::UI;
    use tui::{
        backend::Backend,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        symbols,
        text::{Span, Spans},
        widgets::{Axis, Block, Borders, Chart, Dataset, List, ListItem, Paragraph, Tabs, Wrap},
        Frame,
    };
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
                .map(|t| Spans::from(Span::styled(t.as_str(), Style::default().fg(Color::Green))))
                .collect();
            let tabs = Tabs::new(titles)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(self.config.title.as_str()),
                )
                .highlight_style(Style::default().fg(Color::Yellow))
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
                        Constraint::Length(9),
                        Constraint::Min(8),
                        Constraint::Length(7),
                        Constraint::Percentage(1),
                    ]
                    .as_ref(),
                )
                .split(area);
            self.draw_difference_block(f, chunks[0]);
            self.draw_charts(f, chunks[1]);
            self.draw_legend(f, chunks[2]);
        }

        fn draw_difference_block<B>(&self, f: &mut Frame<B>, area: Rect)
        where
            B: Backend,
        {
            Layout::default()
                .constraints(
                    [
                        Constraint::Length(2),
                        Constraint::Length(3),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .margin(1)
                .split(area);
            let block = Block::default().borders(Borders::ALL).title("Differences");
            let text: Vec<Spans> = self.state.processed_diffs.clone();
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, area);
        }

        fn draw_legend<B>(&self, f: &mut Frame<B>, area: Rect)
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

            let redo = Spans::from(vec![
                Span::from("PageUp: "),
                Span::styled("Redo", Style::default().add_modifier(Modifier::BOLD)),
            ]);

            let undo_redo = Spans::from(vec![
                Span::from("PageDown: "),
                Span::styled("Undo", Style::default().add_modifier(Modifier::BOLD)),
                Span::from(" | "),
                Span::from("PageUp: "),
                Span::styled("Redo", Style::default().add_modifier(Modifier::BOLD)),
            ]);
            let modifier = Spans::from(vec![
                Span::from("s: "),
                Span::styled("modifier", Style::default().add_modifier(Modifier::BOLD)),
            ]);
            let arrow_up_down = Spans::from(vec![
                Span::from("modifier, then arrow_up: "),
                Span::styled("Select Snapshot Pane", Style::default().add_modifier(Modifier::BOLD)),
                Span::from(" | "),
                Span::from("modifier, then arrow_down: "),
                Span::styled("Select Filename Pane", Style::default().add_modifier(Modifier::BOLD)),
            ]);
            let arrow_left_right = Spans::from(vec![
                Span::from("arrow_left: "),
                Span::styled("Decrease Timeslice", Style::default().add_modifier(Modifier::BOLD)),
                Span::from(" | "),
                Span::from("arrow_right: "),
                Span::styled("Increase Timeslice", Style::default().add_modifier(Modifier::BOLD)),
            ]);
            let quit = Spans::from(vec![
                Span::from("q: "),
                Span::styled("Quit", Style::default().add_modifier(Modifier::BOLD)),
            ]);
            let block = Block::default().borders(Borders::ALL).title("Shortcuts");
            let mut text: Vec<Spans> = Vec::new();
            text.push(quit);
            text.push(modifier);
            text.push(undo_redo);
            text.push(arrow_left_right);
            text.push(arrow_up_down);
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, area);
        }

        fn draw_charts<B>(&mut self, f: &mut Frame<B>, area: Rect)
        where
            B: Backend,
        {
            let constraints = if self.config.show_chart {
                vec![Constraint::Percentage(50), Constraint::Percentage(50)]
            } else {
                vec![Constraint::Percentage(100)]
            };
            let chunks = Layout::default()
                .constraints(constraints)
                .direction(Direction::Horizontal)
                .split(area);
            {
                let chunks = Layout::default()
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(chunks[0]);
                {
                    let chunks = Layout::default()
                        .constraints(
                            [Constraint::Percentage(100), Constraint::Percentage(50)].as_ref(),
                        )
                        .direction(Direction::Horizontal)
                        .split(chunks[0]);

                    // Draw tasks
                    let tasks: Vec<ListItem> = self
                        .state
                        .lines
                        .items
                        .iter()
                        .map(|i| ListItem::new(vec![Spans::from(Span::raw(i.as_str()))]))
                        .collect();
                    let tasks = List::new(tasks)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Available Snapshot"),
                        )
                        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                        .highlight_symbol("x ");
                    f.render_stateful_widget(tasks, chunks[0], &mut self.state.lines.state);
                }

                {
                    Layout::default()
                        .constraints(
                            [Constraint::Percentage(100), Constraint::Percentage(50)].as_ref(),
                        )
                        .direction(Direction::Horizontal)
                        .split(chunks[0]);
                }
                let tasks: Vec<ListItem> = self
                    .state
                    .filenames
                    .items
                    .iter()
                    .map(|i| ListItem::new(vec![Spans::from(Span::raw(i.as_str()))]))
                    .collect();
                let tasks = List::new(tasks)
                    .block(Block::default().borders(Borders::ALL).title("Filename"))
                    .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                    .highlight_symbol("x ");
                f.render_stateful_widget(tasks, chunks[1], &mut self.state.filenames.state);
            }
            if self.config.show_chart {
                let x_labels = vec![];
                let datasets = vec![
                    Dataset::default()
                        .name("Legend1")
                        .marker(symbols::Marker::Dot)
                        .style(Style::default().fg(Color::Cyan))
                        .data(&[(34.4, 34.3)]),
                    Dataset::default()
                        .name("Legend2")
                        .marker(symbols::Marker::Dot)
                        .style(Style::default().fg(Color::Yellow))
                        .data(&[(34.4, 34.3)]),
                ];
                let chart = Chart::new(datasets)
                    .block(
                        Block::default()
                            .title(Span::styled(
                                "Hits-Of-Code",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ))
                            .borders(Borders::ALL),
                    )
                    .x_axis(
                        Axis::default()
                            .title("Date")
                            .style(Style::default().fg(Color::Gray))
                            // .bounds(ui.signals.window)
                            .labels(x_labels),
                    )
                    .y_axis(
                        Axis::default()
                            .title("Hits-of-code * 10000")
                            .style(Style::default().fg(Color::Gray))
                            .bounds([0.0, 100.0])
                            .labels(vec![
                                Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                                Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
                            ]),
                    );
                f.render_widget(chart, chunks[1]);
            }
        }
    }
}
