pub mod ui;
mod util;
mod widgets;
use ui::UI;

use std::{
    thread,
    time::{Duration, Instant},
};
pub enum Event<I> {
    Input(I),
    Tick,
}

use std::{error::Error, io::stdout};
use tui::{backend::CrosstermBackend, Terminal};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub fn run(mut ui: UI) -> Result<(), Box<dyn Error>> {
    // init terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Setup input handling
    let tick_rate = Duration::from_millis(300);
    let key_to_ui = ui.communication.key_to_ui.clone();

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    key_to_ui.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                key_to_ui.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    thread::spawn(move || {
        match ui.communication.on_versions.recv() {
            Ok(res) => {
                ui.state.all_versions = res;
                for r in &ui.state.all_versions {
                    ui.state.filenames.add_item(String::from(r.name.clone()));
                    //ui.version_snapshots.add_item(string_to_static_str(String::from(r.datetime.to_string().clone())));
                    // let diffs = r.changes.clone();
                    // for d in diffs {
                    // ui.version_snapshots.add_item(string_to_static_str(d.line));
                    // }
                }
                // ui.title = string_to_static_str(String::from("bar"));
            }
            Err(_) => {}
        }
    });

    loop {
        terminal.draw(|f| ui.draw(f))?;

        match ui.communication.on_key.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char(c) => {
                    ui.on_key(c);
                }
                KeyCode::Up => {
                    ui.on_up();
                }
                KeyCode::Down => {
                    ui.on_down();
                }
                KeyCode::Left => {
                    ui.on_left();
                }
                KeyCode::Right => {
                    ui.on_right();
                }
                KeyCode::Enter => {
                    ui.on_enter();
                }
                _ => {}
            },
            Event::Tick => {
                let h1 = ui.communication.on_lines.try_recv();
                match h1 {
                    Ok(res) => {
                        // todo: value must depend on selected file + timewindow!
                        ui.state.processed_diffs = util::process_new_version(res);
                    }
                    Err(_) => {}
                }
                // ui.on_tick();
            }
        }
        if ui.config.should_quit {
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            disable_raw_mode()?;
            break;
        }
    }

    Ok(())
}
