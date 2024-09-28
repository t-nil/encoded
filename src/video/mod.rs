mod encoder;

use either::Either;
use percentage::{Percentage, PercentageDecimal};
use std::{path::Path, process::ExitStatus};
use thiserror::Error;
use time::{Duration, Time};
use tracing::{info, instrument, warn};

pub const VIDEO_EXTENSIONS: [&str; 2] = [".mp4", ".mkv"];
//#[instrument]
fn is_video_file(path: impl AsRef<Path>) -> bool {
    let Some(filename) = path.as_ref().file_name() else {
        warn!("path has no filename component");
        return false;
    };

    let Some(filename) = filename.to_str() else {
        warn!("path contains non-unicode");
        return false;
    };

    [".mp4", ".mkv"]
        .iter()
        .any(|suffix| filename.ends_with(suffix))
}

pub trait VideoFile {
    fn path(&self) -> &Path;
    fn encode(&self, out: impl AsRef<Path>, params: Encoder) -> impl EncodingFile;
}
pub trait SourceFile: VideoFile {}
pub trait EncodingFile: VideoFile {
    fn source(&self) -> &Path;

    fn start(&mut self);
    fn pause(&mut self);
}

struct EncodingPauseStats {
    paused_times: usize,
    total_pause_duration: Duration,
}

struct EncodingStats {
    finished: Time,
    duration: Duration,
    pause_stats: Option<EncodingPauseStats>,
}

pub trait EncodedFile: VideoFile {
    fn source(&self) -> &Path;

    fn stats(&self) -> EncodingStats;
}
