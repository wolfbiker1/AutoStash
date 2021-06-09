use crate::tui_main::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(app.title))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);
    f.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => draw_first_tab(f, app, chunks[1]),
        1 => draw_second_tab(f, app, chunks[1]),
        2 => draw_third_tab(f, app, chunks[1]),
        _ => {}
    };
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(9),
                Constraint::Min(8),
                Constraint::Length(7),
            ]
            .as_ref(),
        )
        .split(area);
    draw_gauges(f, app, chunks[0]);
    draw_charts(f, app, chunks[1]);
    draw_text(f, chunks[2]);
}

fn draw_gauges<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
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
    let text: Vec<Spans> = app.processed_diffs.clone();
    // let text: Vec<Spans>  = Vec::new();
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_charts<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let constraints = if app.show_chart {
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
                .constraints([Constraint::Percentage(100), Constraint::Percentage(50)].as_ref())
                .direction(Direction::Horizontal)
                .split(chunks[0]);

            // Draw tasks
            let tasks: Vec<ListItem> = app
                .version_snapshots
                .items
                .iter()
                .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
                .collect();
            let tasks = List::new(tasks)
                .block(Block::default().borders(Borders::ALL).title("Available Snapshot"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("x ");
            f.render_stateful_widget(tasks, chunks[0], &mut app.version_snapshots.state);
        }

        {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100), Constraint::Percentage(50)].as_ref())
                .direction(Direction::Horizontal)
                .split(chunks[0]);

            // Draw tasks
            // let tasks: Vec<ListItem> = app
            //     .versions
            //     .items
            //     .iter()
            //     .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
            //     .collect();
            // let tasks = List::new(tasks)
            //     .block(Block::default().borders(Borders::ALL).title("Snapshot"))
            //     .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            //     .highlight_symbol("x ");
            // f.render_stateful_widget(tasks, chunks[0], &mut app.versions.state);
        }
        let tasks: Vec<ListItem> = app
            .filenames
            .items
            .iter()
            .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
            .collect();
        let tasks = List::new(tasks)
            .block(Block::default().borders(Borders::ALL).title("Filename"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("x ");
        // f.render_stateful_widget(tasks, chunks[0], &mut app.versions.state);
        //     let barchart = BarChart::default()
        //         .block(
        //             Block::default()
        //                 .borders(Borders::ALL)
        //                 .title("Noch mehr Statistik"),
        //         )
        //         .bar_width(3)
        //         .bar_gap(2)
        //         .bar_set(symbols::bar::THREE_LEVELS)
        //         .value_style(
        //             Style::default()
        //                 .fg(Color::Black)
        //                 .bg(Color::Green)
        //                 .add_modifier(Modifier::ITALIC),
        //         )
        //         .label_style(Style::default().fg(Color::Yellow))
        //         .bar_style(Style::default().fg(Color::Green));
        f.render_stateful_widget(tasks, chunks[1], &mut app.filenames.state);
    }
    if app.show_chart {
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
                    // .bounds(app.signals.window)
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

fn draw_text<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let text = vec![
        Spans::from(""),
        Spans::from(""),
        Spans::from(vec![
            Span::from(""),
            Span::styled("", Style::default().fg(Color::Red)),
        ]),
        Spans::from(vec![
            Span::raw(""),
            Span::styled("", Style::default().add_modifier(Modifier::ITALIC)),
        ]),
    ];
}

fn draw_second_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);
    let up_style = Style::default().fg(Color::Green);

    let rows = app.servers.iter().map(|s| {
        let style = up_style;
        Row::new(vec![s.name, s.location]).style(style)
    });
    let table = Table::new(rows)
        .header(
            Row::new(vec!["Filename", "Date of Change"])
                .style(Style::default().fg(Color::Yellow))
                .bottom_margin(1),
        )
        .block(Block::default().title("Versions").borders(Borders::ALL))
        .widths(&[
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
        ]);
    f.render_widget(table, chunks[0]);
    let block = Block::default().borders(Borders::ALL).title("Differences");
    f.render_widget(block, chunks[1]);
}

fn draw_third_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);
    let rows = app
        .servers
        .iter()
        .map(|s| Row::new(vec![s.name, s.location]).style(Style::default().fg(Color::Green)));
    let table = Table::new(rows)
        .header(
            Row::new(vec!["Server", "Location", "Status"])
                .style(Style::default().fg(Color::Yellow))
                .bottom_margin(1),
        )
        .block(Block::default().title("Servers").borders(Borders::ALL))
        .widths(&[
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
        ]);
    f.render_widget(table, chunks[0]);
    let block = Block::default().borders(Borders::ALL).title("Differences");
    f.render_widget(block, chunks[1]);
}
