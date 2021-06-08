
use std::sync::mpsc;
use std::{env, process};
use tui;
use std::thread;

fn main() {
    let (tx, rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
    let (tx1, rx1): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

    let t = thread::spawn(|| {
        tui::run_tui(rx, rx1, env::args()).unwrap_or_else(|err| {
            eprintln!("Could not run tui! {:?}", err);
            process::exit(1);
        });
    });

    t.join().unwrap_or_else(|err| {
        eprintln!("Could not join thread {:?}", err);
        process::exit(1);
    });
}
