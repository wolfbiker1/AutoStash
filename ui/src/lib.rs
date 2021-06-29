pub mod ui;
mod util;
mod widgets;
// use crossterm::style::{SetForegroundColor, SetBackgroundColor, ResetColor, Color, Attribute};
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
                        ui.communication.key_to_ui.send(Event::Input(key)).unwrap();
                    }
                }
                if last_tick.elapsed() >= tick_rate {
                    last_tick = Instant::now();
                }
                if ui.communication.on_quit.try_recv().is_ok() {
                    break;
                }
            }
        })
        .unwrap()
}

fn on_versions(ui: Arc<Mutex<UI>>) -> JoinHandle<()> {
    thread::spawn(move || loop {
        let mut ui = ui.lock();
        if let Ok(res) = ui.communication.on_file_versions.try_recv() {
            ui.state.file_versions = res.clone();
            let state = &mut ui.state;
            state.filenames.flush_display();
            res.iter()
                .filter(|file_versions| file_versions.is_some())
                .for_each(|file_versions| {
                    state
                        .filenames
                        .add_item(file_versions.as_ref().unwrap().path.clone());
                });
            ui.state.update_pane_content();
        }
        if ui.communication.on_quit.try_recv().is_ok() {
            break;
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

            if let Ok(ev) = ui.communication.on_key.try_recv() {
                match ev {
                    Event::Input(ev) => match ev.code {
                        KeyCode::Char(c) => {
                            ui.state.on_key(c);
                        }
                        // TODO
                        KeyCode::Esc => {
                            let selected_path = ui.state.path_of_selected_file.clone();
                            ui.communication.on_undo(selected_path, 1);
                        }
                        // TODO
                        KeyCode::Tab => {
                            let selected_path = ui.state.path_of_selected_file.clone();
                            ui.communication.on_redo(selected_path, 1);
                        }
                        KeyCode::Up => {
                            ui.state.on_up();
                        }
                        KeyCode::Down => {
                            ui.state.on_down();
                        }
                        KeyCode::Left => {
                            ui.state.on_left();
                            let current_id = ui.state.tabs.get_index();
                            ui.communication.on_timeslice_change(current_id);
                        }
                        KeyCode::Right => {
                            ui.state.on_right();
                            let current_id = ui.state.tabs.get_index();
                            ui.communication.on_timeslice_change(current_id);
                        }
                        _ => {}
                    },
                    Event::Tick => {}
                }
            }
            if ui.communication.on_quit.try_recv().is_ok() {
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

///
/// sets up crossterm - terminal, inits event listeners,
/// starts the rx-listeners, draws the terminal
/// in an infinite loop
///
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
