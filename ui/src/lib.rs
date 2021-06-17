pub mod ui;
mod util;
mod widgets;
use std::sync::Arc;
use ui::UI;
use parking_lot::Mutex;

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

pub fn run(ui: UI) -> Result<(), Box<dyn Error>> {
    // init terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Setup input handling
    let tick_rate = Duration::from_millis(300);

    let ui = Arc::new(Mutex::new(ui));
    let ui_cloned = ui.clone();

    
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        
        loop {
            let ui = ui_cloned.lock();
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    ui.communication.key_to_ui.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                ui.communication.key_to_ui.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });
    

    let ui_cloned = ui.clone();

    thread::spawn(move || {
        let mut ui = ui_cloned.lock();
        println!("reached here");
        loop {
            match ui.communication.on_versions.recv() {
                Ok(res) => {
                    ui.state.all_versions = res;
                    // Non-lexical borrows don't exist in rust yet
                    let state = &mut ui.state;
                    let versions = &mut state.all_versions;
                    let filenames = &mut state.filenames;
                    versions.iter().for_each(|v| {
                        filenames.add_item(String::from(v.name.clone()));
                        //ui.version_snapshots.add_item(string_to_static_str(String::from(r.datetime.to_string().clone())));
                        // let diffs = r.changes.clone();
                        // for d in diffs {
                        // ui.version_snapshots.add_item(string_to_static_str(d.line));
                        // }
                    });
                    // ui.title = string_to_static_str(String::from("bar"));
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
    });

    
    let ui = ui.clone();
    
    loop {
        let mut ui = ui.lock();
        terminal.draw(|f| ui.draw(f))?;

        if let Ok(on_key) = ui.communication.on_key.try_recv() {
            match on_key {
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
