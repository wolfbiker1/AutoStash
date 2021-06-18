pub mod ui;
mod util;
mod widgets;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use parking_lot::{Mutex, MutexGuard};
use std::{
    error::Error,
    io::stdout,
    io::Stdout,
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};
use ui::UI;

pub enum Event<I> {
    Input(I),
    Tick,
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    Ok(terminal)
}

fn listen_to_key_press(ui: Arc<Mutex<UI>>, tick_rate: Duration) -> JoinHandle<()> {
    thread::Builder::new()
        .name("key_press".to_string())
        .spawn(move || {
            let mut last_tick = Instant::now();

            loop {
                let ui = ui.lock();
                // poll for tick rate duration, if no events, sent tick event.
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                if event::poll(timeout).unwrap() {
                    if let CEvent::Key(key) = event::read().unwrap() {
                        println!("{:?}", key);
                        ui.communication.key_to_ui.send(Event::Input(key)).unwrap();
                    }
                }
                if last_tick.elapsed() >= tick_rate {
                    //ui.communication.key_to_ui.send(Event::Tick).unwrap();
                    last_tick = Instant::now();
                }
                if let Ok(_) = ui.communication.on_quit.try_recv() {
                    break;
                }
            }
        })
        .unwrap()
}

fn on_versions(ui: Arc<Mutex<UI>>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let mut ui = ui.lock();
            match ui.communication.on_versions.try_recv() {
                Ok(res) => {
                    ui.state.all_versions = res;
                    // Non-lexical borrows don't exist in rust yet
                    let state = &mut ui.state;
                    let versions = &mut state.all_versions;
                    let filenames = &mut state.filenames;
                    versions.iter().for_each(|v| {
                        filenames.add_item(v.name.clone());
                        //ui.version_snapshots.add_item(string_to_static_str(String::from(r.datetime.to_string().clone())));
                        // let diffs = r.changes.clone();
                        // for d in diffs {
                        // ui.version_snapshots.add_item(string_to_static_str(d.line));
                        // }
                    });
                    // ui.title = string_to_static_str(String::from("bar"));
                }
                Err(e) => {
                    //println!("{:?}", e);
                }
            }

            if let Ok(_) = ui.communication.on_quit.try_recv() {
                break;
            }
        }
    })
}

fn draw(
    ui: Arc<Mutex<UI>>,
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
    handles: Vec<JoinHandle<()>>,
) -> Result<(), Box<dyn Error>> {
    loop {
        let mut ui = ui.lock();
        terminal.draw(|f| ui.draw(f))?;

        if ui.state.should_quit {
            quit(terminal)?;

            ui.communication.quit_to_handle.send(()).unwrap();
            handles.iter().for_each(|_| {
                ui.communication.quit_to_ui.send(()).unwrap();
            });
            MutexGuard::unlock_fair(ui);
            for handle in handles {
                handle.join().unwrap();
            }
            break;
        }
    }

    Ok(())
}

fn on_key(ui: Arc<Mutex<UI>>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let mut ui = ui.lock();
            match ui.communication.on_key.try_recv() {
                Ok(ev) => {
                    match ev {
                        Event::Input(ev) => match ev.code {
                            KeyCode::Char(c) => {
                                ui.state.on_key(c);
                            }
                            KeyCode::Up => {
                                ui.state.on_up();
                            }
                            KeyCode::Down => {
                                ui.state.on_down();
                            }
                            KeyCode::Left => {
                                ui.state.on_left();
                            }
                            KeyCode::Right => {
                                ui.state.on_right();
                            }
                            KeyCode::Enter => {
                                ui.state.on_enter();
                            }
                            _ => {}
                        },
                        Event::Tick => {
                            let h1 = ui.communication.on_lines.try_recv();
                            if let Ok(res) = h1 {
                                // todo: value must depend on selected file + timewindow!
                                ui.state.processed_diffs = util::process_new_version(res);
                            }
                        }
                    }
                }
                Err(_) => (),
            }
            if let Ok(_) = ui.communication.on_quit.try_recv() {
                break;
            }
        }
    })
}

fn quit(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    disable_raw_mode()?;

    Ok(())
}

pub fn run(ui: UI) -> Result<(), Box<dyn Error>> {
    let terminal = init_terminal()?;
    let tick_rate = Duration::from_millis(300);
    let ui = Arc::new(Mutex::new(ui));

    let handles = vec![
        listen_to_key_press(ui.clone(), tick_rate),
        on_versions(ui.clone()),
        on_key(ui.clone()),
    ];
    draw(ui, terminal, handles)
}
