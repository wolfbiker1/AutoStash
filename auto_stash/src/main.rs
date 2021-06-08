use std::sync::mpsc;
use std::thread;
use std::{env, process};
use tui;

fn main() {
    let t = thread::spawn(|| {
        tui::run_tui(env::args()).unwrap_or_else(|err| {
            eprintln!("Could not run tui! {:?}", err);
            process::exit(1);
        });
    });

    t.join().unwrap_or_else(|err| {
        eprintln!("Could not join thread {:?}", err);
        process::exit(1);
    });
}
