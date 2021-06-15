use std::{env, process, sync::mpsc, thread};

use auto_stash::{AutoStash, Config};
use diff::LineDifference;
use event_handle::event_handle::EventHandleCommunication;
use store::store::Version;
use ui::ui::{UICommunication, UI};

fn main() {
    let (versions_to_ui, on_versions): (mpsc::Sender<Vec<Version>>, mpsc::Receiver<Vec<Version>>) =
        mpsc::channel();
    let (lines_to_ui, on_lines): (
        mpsc::Sender<Vec<LineDifference>>,
        mpsc::Receiver<Vec<LineDifference>>,
    ) = mpsc::channel();
    let (undo_to_handle, on_undo): (mpsc::Sender<usize>, mpsc::Receiver<usize>) = mpsc::channel();
    let (redo_to_handle, on_redo): (mpsc::Sender<usize>, mpsc::Receiver<usize>) = mpsc::channel();
    let (key_to_ui, on_key) = mpsc::channel();

    let ui = UI::new(
        "".to_string(),
        UICommunication {
            on_key,
            on_lines,
            on_versions,
            key_to_ui,
            redo_to_handle,
            undo_to_handle,
        },
    );

    thread::spawn(|| {
        ui::run(ui).unwrap_or_else(|err| {
            eprintln!("Could not run ui: {:?}", err);
            process::exit(1);
        });
    });

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let mut auto_stash = AutoStash::new(
        &config,
        EventHandleCommunication {
            lines_to_ui,
            versions_to_ui,
            on_redo,
            on_undo,
        },
    )
    .unwrap_or_else(|err| {
        eprintln!("Problem creating auto stash: {:?}", err);
        process::exit(1);
    });

    auto_stash.run().unwrap_or_else(|err| {
        eprintln!("Could not run auto stash: {:?}", err);
        process::exit(1);
    });
}
