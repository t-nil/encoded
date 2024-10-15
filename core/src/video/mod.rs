pub mod encoder;
pub mod preview;
pub mod stats;
pub mod zones;

use chrono::{DateTime, Duration, Local};
use encoder::Encoder;
use std::{path::Path, process::ExitStatus};
use thiserror::Error;
use tracing::{info, instrument, warn};

use typestate::typestate;

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

#[typestate]
mod video_file {
    use super::stats::EncodedStats;

    #[automaton]
    pub struct VideoFile;

    #[state]
    pub struct Detected;
    /*#[state]
    pub struct ReadyForEncoding;
    #[state]
    pub struct Encoding;*/
    #[state]
    pub struct Completed;

    #[state]
    pub struct Errored;

    pub enum EncodingResult {
        Completed,
        Errored,
    }

    /// initial state
    pub trait Detected {
        fn new() -> Detected {
            todo!()
        }

        fn maybe_completed(self) -> EncodingResult
        where
            Self: Sized,
        {
            todo!()
        }
    }

    pub trait Errored {
        fn reap(self) -> color_eyre::Report;
    }

    pub trait Completed {
        fn to_stats(self) -> EncodedStats
        where
            Self: Sized,
        {
            todo!()
        }
    }
}

pub trait VideoFile {
    fn path(&self) -> &Path;
    fn encode(&self, out: impl AsRef<Path>, params: &dyn Encoder) -> impl EncodingFile;
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
    finished: DateTime<Local>,
    duration: Duration,
    pause_stats: Option<EncodingPauseStats>,
}

pub trait EncodedFile: VideoFile {
    fn source(&self) -> &Path;

    fn stats(&self) -> EncodingStats;
}
