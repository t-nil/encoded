#![feature(step_trait)]

mod check;
mod cli;
mod tags;
mod video;

use crate::cli::Args;
use clap::Parser as _;
use color_eyre::Result;
use glob::{glob, GlobResult};
use itertools::Itertools as _;
use notify::poll::ScanEvent;
use notify::{Config, PollWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{error, warn};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let raw_path = args
        .raw_path
        .unwrap_or_else(|| args.encoded_path.join("!raw"));

    check_requirements()?;
    Ok(())
}

/// scans only depth 1
fn scan_raw(path: impl AsRef<Path>, vec: &mut Vec<PathBuf>) {
    vec.clear();
    let paths = video::VIDEO_EXTENSIONS
        .iter()
        .flat_map(|ext| {
            glob((path.as_ref().to_str().unwrap().to_owned() + "/*" + ext).as_str()).unwrap()
        })
        .filter_map(|glob_result| match glob_result {
            Ok(path) => Some(path),
            Err(e) => {
                let e = e.to_string();
                warn!(error = e, "while globbing");
                None
            }
        });
    vec.extend(paths)
}

fn check_requirements() -> Result<()> {
    use check::*;

    check_for_exe("steamcmd")?;
    check_for_exe("ffmpeg")?;

    Ok(())
}

fn watch(args: &Args) -> Result<()> {
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    enum Message {
        Event(notify::Result<notify::Event>),
        Scan(ScanEvent),
    }

    let tx_c = tx.clone();
    // use the pollwatcher and set a callback for the scanning events
    let mut watcher = PollWatcher::with_initial_scan(
        move |watch_event| {
            tx_c.send(Message::Event(watch_event)).unwrap();
        },
        Config::default().with_poll_interval(Duration::from_secs(5)),
        move |scan_event| {
            tx.send(Message::Scan(scan_event)).unwrap();
        },
    )?;

    watcher.watch(&args.encoded_path, RecursiveMode::NonRecursive)?;
    watcher.watch(raw_path, RecursiveMode::NonRecursive)?;

    let (mut raw_vids, mut encoded_vids) = (Vec::new(), Vec::new());

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.

    for res in rx {
        match res {
            Message::Event(e) => {
                let event = match e {
                    Ok(event) => event,
                    Err(err) => {
                        let err = err.to_string();
                        error!(error = err, "Processing notify event");
                        continue;
                    }
                };
                if event.need_rescan() {
                    todo!()
                }
            }
            Message::Scan(e) => {
                let path = match e {
                    Ok(path) => path,
                    Err(err) => {
                        let err = err.to_string();
                        error!(error = err, "Processing notify event");
                        continue;
                    }
                };
                match path {
                    p if p == args.encoded_path => {}
                    p if p == raw_path => scan_raw(raw_path, &mut raw_vids),
                    _ => unreachable!(), // Means we scanned a path in the init code at the start of the function,
                                         // which doesn't appear in the match here. That's a logic error.
                }
            }
        }
    }

    todo!()
}
