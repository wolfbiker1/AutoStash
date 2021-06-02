use std::{env, process};

use auto_stash::{AutoStash, Config};

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let mut auto_stash = AutoStash::new(&config).unwrap_or_else(|err| {
        eprintln!("Problem creating auto stash: {:?}", err);
        process::exit(1);
    });

    auto_stash.run().unwrap_or_else(|err| {
        eprintln!("Could not run auto stash: {:?}", err);
    });
}
