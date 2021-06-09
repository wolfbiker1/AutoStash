mod tui_main;
#[allow(dead_code)]
mod util;
use crate::tui_main::{ui, App, AutoStash, Config};
use diff::LineDifference;
use std::io;
use std::process;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

use std::time::{Duration, Instant};
pub enum Event<I> {
    Input(I),
    Tick,
}

// extern   crate auto_
/// type is handled in its own thread and returned to a common `Receiver`
// pub struct Events {
//     rx: mpsc::Receiver<Event<Key>>,
//     ignore_exit_key: Arc<AtomicBool>,
// }

// impl Events {
//     pub fn new() -> Events {
//         Events::with_config()
//     }

//     pub fn with_config() -> Events {
//         let (tx, rx) = mpsc::channel();
//         let ignore_exit_key = Arc::new(AtomicBool::new(false));
//         {
//             let tx = tx.clone();
//             thread::spawn(move || {
//                 let stdin = io::stdin();
//                 for evt in stdin.Key() {
//                     if let Ok(key) = evt {
//                         if let Err(err) = tx.send(Event::Input(key)) {
//                             eprintln!("{}", err);
//                             return;
//                         }
//                     }
//                 }
//             })

//         };
//         let _tick_handle = {
//             thread::spawn(move || loop {
//                 if tx.send(Event::Tick).is_err() {
//                     break;
//                 }
//                 thread::sleep(Duration::from_millis(500));
//             })
//         };
//         Events {
//             rx,
//             ignore_exit_key
//         }
//     }

//     pub fn next(&self) -> Result<Event<KeyCode>, mpsc::RecvError> {
//         self.rx.recv()
//     }

//     pub fn disable_exit_key(&mut self) {
//         self.ignore_exit_key.store(true, Ordering::Relaxed);
//     }

//     pub fn enable_exit_key(&mut self) {
//         self.ignore_exit_key.store(false, Ordering::Relaxed);
//     }
// }

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

use std::{error::Error, io::stdout};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub fn run_tui(args: std::env::Args) -> Result<(), Box<dyn Error>> {
    let (tx, rx1): (
        mpsc::Sender<Vec<LineDifference>>,
        mpsc::Receiver<Vec<LineDifference>>,
    ) = mpsc::channel();
    let (tx1, rx_new_version): (
        mpsc::Sender<Vec<LineDifference>>,
        mpsc::Receiver<Vec<LineDifference>>,
    ) = mpsc::channel();

    let (undo_redo_tx, undo_redo_rx): (
        mpsc::Sender<(u8, u8)>,
        mpsc::Receiver<(u8, u8)>,
    ) = mpsc::channel();

    // let events = Events::with_config();
    // let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    // let stdout = AlternateScreen::from(stdout);

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let config = Config::new(args).unwrap();
    let mut auto_stash = AutoStash::new(&config, tx, tx1, undo_redo_rx).unwrap();
    let app = App::new("AutoStash");

    let mut app = app.unwrap();

    thread::spawn(move || {
        auto_stash.run().unwrap_or_else(|err| {
            eprintln!("Could not join thread {:?}", err);
            process::exit(1);
        });
    });

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(300);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });
    terminal.clear()?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let h = rx.try_recv();
        match h {
            Ok(res) => {}
            Err(_) => {}
        }

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char(c) => {
                    app.on_key(c);
                }
                KeyCode::Up => {
                    app.on_up();
                }
                KeyCode::Down => {
                    app.on_down();
                }
                KeyCode::Left => {
                    app.on_left();
                }
                KeyCode::Right => {
                    app.on_right();
                }
                _ => {}
            },
            Event::Tick => {
                let h1 = rx_new_version.try_recv();
                match h1 {
                    Ok(res) => {
                        // todo: value must depend on selected file + timewindow!
                        app.processed_diffs = util::process_new_version(res);
                    }
                    Err(_) => {}
                }
                // app.on_tick();
            }
        }
        if app.should_quit {
            disable_raw_mode()?;
            break;
        }
    }

    Ok(())
}
