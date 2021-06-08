mod tui_main;
#[allow(dead_code)]
mod util;

use crate::{
    tui_main::{ui, App, Config, AutoStash}
};

use std::env::Args;

use std::io;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}


// extern crate auto_
/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    ignore_exit_key: Arc<AtomicBool>,
}


impl Events {
    pub fn new() -> Events {
        Events::with_config()
    }

    pub fn with_config() -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                    }
                }
            })
        };
        Events {
            rx,
            ignore_exit_key
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}


fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

use std::{error::Error};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
// use tui::{backend::TermionBackend, backend::CrosstermBackend, Terminal};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
  };
pub fn run_tui(args: std::env::Args) -> Result<(), Box<dyn Error>> {
    let (tx, rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
    let (tx1, rx1): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

    let events = Events::with_config();
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let config = Config::new(args).unwrap();
    let mut f = AutoStash::new(&config, tx, tx1).unwrap();
    let app = App::new("AutoStash");

    let mut app = app.unwrap();
    
    let f1 = thread::spawn(move || {
        // app.watch.start_watching(app.watch_path.as_str())
        f.run();
    });



    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let h = stack_transmitter.try_recv();

        // app.title = h.unwrap_err();
        match h {
            Ok(res) => {
                app.title = string_to_static_str(res);
                
            }
            Err(_) => {}
        }


        let h1 = version_transmitter.try_recv();

        // app.title = h.unwrap_err();
        match h1 {
            Ok(res) => {
                app.title = string_to_static_str(res);
                
            }
            Err(_) => {}
        }
        match events.next()? {
            Event::Input(key) => match key {
                Key::Char(c) => {
                    app.on_key(c);
                }
                Key::Up => {
                    app.title = "d";
                    app.on_up();
                }
                Key::Down => {
                    app.on_down();
                }
                Key::Left => {
                    app.on_left();
                }
                Key::Right => {
                    app.on_right();
                }
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
