#![feature(step_trait)]

mod check;
mod cli;
pub mod ffmpeg;
pub mod video;
pub mod fs;
pub mod tags;
pub mod util;

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

fn scan(path: impl AsRef<Path>, vec: &mut Vec<PathBuf>) {
    vec.clear();
    let paths = ffmpeg::VIDEO_EXTENSIONS
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
