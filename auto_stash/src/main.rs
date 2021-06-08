// use std::thread;
pub use auto_stash::{AutoStash, Config};
use std::io;
use std::sync::mpsc;
use std::{env, process};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui;
// use tui::{backend::TermionBackend, Terminal};
use std::thread;

fn main() {
    let stdout = io::stdout().into_raw_mode();
    // let stdout = MouseTerminal::from(stdout);
    // let stdout = AlternateScreen::from(stdout);
    // let backend = TermionBackend::new(stdout);
    // let mut terminal = Terminal::new(backend)?;

    let (tx, rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
    let (tx1, rx1): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let mut auto_stash = AutoStash::new(&config, tx, tx1).unwrap_or_else(|err| {
        eprintln!("Problem creating auto stash: {:?}", err);
        process::exit(1);
    });

    let t = thread::spawn(|| {
        tui::run_tui(rx, rx1, &config).unwrap_or_else(|err| {
            eprintln!("Could not run tui! {:?}", err);
            process::exit(1);
        });
    });
    // let a = thread::spawn(move || {
    //     auto_stash.run().unwrap_or_else(|err| {
    //         eprintln!("Could not run auto stash: {:?}", err);
    //         process::exit(1);
    //     });
    // });

    t.join().unwrap_or_else(|err| {
        eprintln!("Could not join thread {:?}", err);
        process::exit(1);
    });
    // a.join().unwrap_or_else(|err| {
    //     eprintln!("Could not join thread {:?}", err);
    //     process::exit(1);
    // });
}
