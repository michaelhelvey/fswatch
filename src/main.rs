use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

use anyhow::{Context, Result};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use regex::Regex;

#[derive(Parser, Debug)]
#[clap(
    version,
    about = "Watches a file path for changes; runs a command on those changes"
)]
struct Args {
    /// File or directory to watch for changes.  Glob patterns are not supported: use --exclude if
    /// you need to filter changes in a directory.  If file path isn't unicode good luck.
    file_path: String,

    /// Command to run when the filepath changes
    command: Vec<String>,

    /// Interval in seconds used to debounce file change events
    #[clap(short, long, default_value_t = 2)]
    debounce_interval: i32,

    /// Regex pattern to exclude from watch
    #[clap(short, long)]
    exclude: Option<String>,
}

// filters event and exclude regex to determine if the command needs to be run
fn should_run_command(event: &DebouncedEvent, exclude: &Option<Regex>) -> bool {
    // convert our path to a unicode string (not handling non-unicode chars), then match it against
    // our regex, if the regex exists, otherwise returning true if we don't have an exclusion
    // regex.
    let matcher = |path: &PathBuf| -> bool {
        exclude
            .as_ref()
            .map(|regex| !regex.is_match(&path.to_string_lossy()))
            .unwrap_or(true)
    };

    match event {
        DebouncedEvent::NoticeWrite(path) => matcher(path),
        DebouncedEvent::Create(path) => matcher(path),
        DebouncedEvent::NoticeRemove(path) => matcher(path),
        DebouncedEvent::Rename(_, path) => matcher(path),
        _ => false,
    }
}

fn handle_file_change(event: DebouncedEvent, command: &[String], exclude: &Option<Regex>) {
    let run_command = should_run_command(&event, exclude);

    if run_command {
        println!("fswatch: ChangeEvent: {:?}", &event);
        let result = Command::new(&command[0]).args(&command[1..]).spawn();

        match result {
            Ok(..) => {}
            Err(e) => eprintln!("fswatch: {} failed with error {}", command.join(" "), e),
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // if we receive a regex, compile it, and exit if compilation fails.  Otherwise pass through
    // the empty option to our handler
    let exclude_regex = match args.exclude {
        Some(pattern) => Some(
            Regex::new(&pattern)
                .context("Could not compile regular expression for 'exclude' argument")?,
        ),
        None => None,
    };

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch(&args.file_path, RecursiveMode::Recursive)
        .with_context(|| format!("Unable to watch filepath {}", &args.file_path))?;

    println!("fswatch: watching {} for changes...", args.file_path);
    // loop forever watching for change events
    loop {
        match rx.recv() {
            Ok(event) => handle_file_change(event, &args.command, &exclude_regex),
            Err(e) => println!("fswatch: watcher error from channel: {:?}", e),
        }
    }
}
