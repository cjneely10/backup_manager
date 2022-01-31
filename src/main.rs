extern crate notify;

use notify::{watcher, RecursiveMode, Watcher};
use std::fs::canonicalize;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    let args: Vec<PathBuf> = std::env::args().map(|v| canonicalize(v).unwrap()).collect();
    assert_eq!(args.len(), 2, "Usage: backup_manager <path>");
    println!("{:?}", args);
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch(args.get(1).unwrap(), RecursiveMode::Recursive)
        .unwrap();

    loop {
        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
