use std::thread;
use std::{env, process};

use auto_stash::{AutoStash, Config};
use tui;
fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let mut auto_stash = AutoStash::new(&config).unwrap_or_else(|err| {
        eprintln!("Problem creating auto stash: {:?}", err);
        process::exit(1);
    });

    // let a = thread::spawn(move || {
    //     auto_stash.run().unwrap_or_else(|err| {
    //         eprintln!("Could not run auto stash: {:?}", err);
    //         process::exit(1);
    //     });
    // });

    let t = thread::spawn(|| {
        tui::run_tui().unwrap_or_else(|err| {
            eprintln!("Could not run tui! {:?}", err);
            process::exit(1);
        });
    });

    t.join().unwrap_or_else(|err| {
        eprintln!("Could not join thread {:?}", err);
        process::exit(1);
    });
    // a.join().unwrap_or_else(|err| {
    //     eprintln!("Could not join thread {:?}", err);
    //     process::exit(1);
    // });
}
