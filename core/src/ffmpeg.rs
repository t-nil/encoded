use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::Stdio,
};

use argument::Argument;
use atomic_write_file::AtomicWriteFile;
use chrono::Duration;
use color_eyre::{eyre::bail, Result};
use derive_more::derive::{Add, Into, Mul};
use itertools::Itertools as _;
use regex::Regex;
use serde::{Deserialize, Serialize};
use size::Size;
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{ChildStderr, ChildStdout, Command},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::error;

pub const VIDEO_EXTENSIONS: [&str; 2] = [".mp4", ".mkv"];

#[derive(Debug, Clone, Add, Mul, PartialEq, PartialOrd, Into)]
struct Bitrate {
    kilobits_per_second: f64,
}

impl Bitrate {
    pub fn new(kilobits: impl AsRef<Size>, seconds: Duration) -> Self {
        let kilobits = kilobits.as_ref().bytes() as f64 / 1024. * 8.;
        let seconds = seconds.num_seconds();
        Self {
            kilobits_per_second: (kilobits / (seconds as f64)).into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EncodeProgress {
    fps: Option<f32>,
    speed: Option<f32>,
    time: Duration,
    frame: Option<usize>,
    size: Option<Size>,
    bitrate: Option<Bitrate>,
}

#[derive(Debug, Clone)]
pub enum EncodeStatusUpdate {
    Progress(EncodeProgress),
    Finished,
    Cancelled,
    Error(Vec<String>),
}

pub struct Handle {
    pub stderr: JoinHandle<Result<()>>,
    pub stdout: JoinHandle<Result<()>>,
    pub channel: UnboundedReceiver<EncodeStatusUpdate>,
}
pub mod argument {
    use std::path::PathBuf;

    use itertools::Itertools as _;
    use thiserror::Error;
    use tokio::fs::File;

    /// Represents an FFmpeg output (with temporary redirect and such).
    /// - `Closed`: should be the "default". Output is set but processing has not started.
    /// - `Open`: Processing has started. Temporary file is open and _will be dropped_ when this struct goes out of scope.
    /// - `Empty`: After cloning or if you don't want to set an output right away. Saves last path
    #[derive(Debug)]
    pub enum OutputFile {
        Empty(Option<PathBuf>),
        Closed(PathBuf),
        Open(File),
    }

    /// This works as one'd expect, with the exception that two `Open` files always compare false, even if they had the same file open (which they shouldn't!!)
    impl PartialEq for OutputFile {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Empty(l0), Self::Empty(r0)) => l0 == r0,
                (Self::Closed(l0), Self::Closed(r0)) => l0 == r0,
                (Self::Open(_), Self::Open(_)) => false,
                _ => false,
            }
        }
    }

    /// Clone always replaces a file with an `Empty` variant. Closed->Empty saves the path_buf inside the option, Open->Empty uses None
    impl Clone for OutputFile {
        fn clone(&self) -> Self {
            match self {
                Self::Closed(path_buf) => Self::Empty(Some(path_buf.clone())),
                Self::Open(_) => Self::Empty(None),
                Self::Empty(path_buf_opt) => Self::Empty(path_buf_opt.clone()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Argument {
        InputFile(PathBuf),
        OutputFile(OutputFile),
        Flag(String),
        Parameter { key: String, val: String },
    }

    #[derive(Debug, Error, PartialEq)]
    pub enum IntoStringError {
        #[error("One or more outputs are not set. (Previous path: {0:?})")]
        OutputNotSet(Option<PathBuf>),
        #[error("Non-unicode chars found inside a path: {0:?}")]
        PathHasNonUnicode(PathBuf),
        #[error("The encode has already been started, as there are open output files")]
        EncodeAlreadyRunning,
        #[error("Input file does not exist: {0:?}")]
        NonExistingInput(PathBuf),
        #[error("Output file already exists: {0:?}")]
        OutputExists(PathBuf),
    }
    impl Argument {
        pub fn try_into_string(self) -> Result<Vec<String>, IntoStringError> {
            use IntoStringError::*;
            Ok(match self {
                Argument::InputFile(input) => {
                    if input.exists() {
                        vec![
                            "-i".to_owned(),
                            input
                                .to_str()
                                .ok_or_else(|| IntoStringError::PathHasNonUnicode(input.clone()))?
                                .to_owned(),
                        ]
                    } else {
                        return Err(NonExistingInput(input));
                    }
                }
                Argument::OutputFile(output) => match output {
                    OutputFile::Empty(path_buf_opt) => {
                        return Err(IntoStringError::OutputNotSet(path_buf_opt))
                    }
                    OutputFile::Closed(output) => vec![(!output.exists())
                        .then_some(output.to_str())
                        .ok_or_else(|| IntoStringError::OutputExists(output.clone()))?
                        .ok_or_else(|| IntoStringError::PathHasNonUnicode(output.clone()))?
                        .to_owned()],
                    OutputFile::Open(_) => return Err(IntoStringError::EncodeAlreadyRunning),
                },
                Argument::Flag(flag) => vec![flag],
                Argument::Parameter { key, val } => vec![key, val],
            })
        }

        /// order is important, therefor preserved
        pub fn try_vec_into_strings(args: Vec<Argument>) -> Result<Vec<String>, IntoStringError> {
            //let x = args.into_iter()
            //  .map(Argument::try_into_string).collect_vec(); x.into_iter().process_results(|vec| vec.)
            todo!()
        }
    }

    #[allow(non_snake_case)]
    #[cfg(test)]
    mod test {
        use std::{path::Path, process::Command};

        use color_eyre::Result;
        use decl_macros::apply_common_filters;
        use insta::assert_debug_snapshot;
        use tempfile::{tempdir, tempfile, tempfile_in, NamedTempFile};
        use uuid::Uuid;

        use super::*;
        #[test]
        fn Argument__basic() -> Result<()> {
            let tempdir = tempdir()?;
            let input_file = tempdir.path().join("input.flv");
            Command::new("touch")
                .arg(tempdir.path().join("input.flv"))
                .output()?;
            apply_common_filters!();
            assert_debug_snapshot!([
                Argument::Flag("-hide_banner".into()),
                Argument::InputFile(input_file),
                Argument::Parameter {
                    key: "-codec".into(),
                    val: "copy".into(),
                },
                Argument::OutputFile(OutputFile::Closed(tempdir.path().join("foo.mkv"))),
            ]
            .into_iter()
            .map(|arg| arg.try_into_string())
            .collect_vec(), @r###"
            [
                Ok(
                    [
                        "-hide_banner",
                    ],
                ),
                Ok(
                    [
                        "-i",
                        "[TEMP_FILE]/input.flv",
                    ],
                ),
                Ok(
                    [
                        "-codec",
                        "copy",
                    ],
                ),
                Ok(
                    [
                        "[TEMP_FILE]/foo.mkv",
                    ],
                ),
            ]
            "###);
            Ok(())
        }

        #[test]
        fn try_into_string__errors() -> Result<()> {
            use IntoStringError::*;

            let temp_dir = tempdir()?;
            let existing_file = tempfile::NamedTempFile::new()?;
            let non_existing_file = PathBuf::from(format!("/foo/bar/{}", Uuid::new_v4()));

            assert_eq!(
                Argument::OutputFile(OutputFile::Empty(None)).try_into_string(),
                Err(OutputNotSet(None))
            );
            assert_eq!(
                Argument::OutputFile(OutputFile::Empty(Some(non_existing_file.clone())))
                    .try_into_string(),
                Err(OutputNotSet(Some(non_existing_file.clone())))
            );
            assert_eq!(
                Argument::OutputFile(OutputFile::Closed(existing_file.path().to_path_buf()))
                    .try_into_string(),
                Err(OutputExists(existing_file.path().to_path_buf()))
            );
            assert_eq!(
                Argument::OutputFile(OutputFile::Open(existing_file.into_file().into()))
                    .try_into_string(),
                Err(EncodeAlreadyRunning)
            );

            assert_eq!(
                Argument::InputFile(non_existing_file.clone()).try_into_string(),
                Err(NonExistingInput(non_existing_file.clone()))
            );

            // only works on linux currently - see more on `test_non_utf8` source.
            // FIXME this dang `i_only_find_the_old_crate_version_herpaderpaderp` rustc >___<
            use decl_macros::test::insta::generate_non_utf8_path;
            let non_utf8_path = generate_non_utf8_path();
            assert!(non_utf8_path.is_relative()); // IMPORTANT we don't want to accidentally create that file anywhere
            let non_utf8_existing = NamedTempFile::with_suffix(non_utf8_path)?;
            assert_eq!(
                Argument::InputFile(non_utf8_existing.path().to_path_buf()).try_into_string(),
                Err(PathHasNonUnicode(non_utf8_existing.path().to_path_buf()))
            );
            Ok(())
        }

        #[test]
        fn OutputFile__Clone_() -> Result<()> {
            let path_buf = PathBuf::from("/foo/bar/baz");
            assert_eq!(
                OutputFile::Closed(path_buf.clone()).clone(),
                OutputFile::Empty(Some(path_buf.clone()))
            );
            let temp_file = tempfile()?;
            assert_eq!(
                OutputFile::Open(temp_file.into()).clone(),
                OutputFile::Empty(None)
            );
            Ok(())
        }

        #[test]
        fn OutputFile__PartialEq() -> Result<()> {
            let temp_path = Path::new("foo/bar/baz");
            let (open, closed, empty_some, empty_none) = (
                OutputFile::Open(tempfile()?.into()),
                OutputFile::Closed(temp_path.to_path_buf()),
                OutputFile::Empty(Some(temp_path.to_path_buf())),
                OutputFile::Empty(None),
            );

            assert_ne!(open, open);
            assert_eq!(closed, closed);
            assert_eq!(empty_some, empty_some);
            assert_eq!(empty_none, empty_none);

            assert_ne!(open, closed);
            assert_ne!(open, empty_some);
            assert_ne!(open, empty_none);
            assert_ne!(closed, open);
            assert_ne!(closed, empty_some);
            assert_ne!(closed, empty_none);
            assert_ne!(empty_some, open);
            assert_ne!(empty_some, closed);
            assert_ne!(empty_some, empty_none);
            assert_ne!(empty_none, open);
            assert_ne!(empty_none, closed);
            assert_ne!(empty_none, empty_some);

            Ok(())
        }
    }
}

/// Consumes because of the notion that output file instances are consumed.
async fn encode(args: Vec<Argument>) -> Result<Handle> {
    let (tx, rx) = mpsc::unbounded_channel();

    let mut child = Command::new("ffmpeg")
        .args(Argument::try_vec_into_strings(args)?)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    /*let output_files = args
    .iter()
    .filter_map(|arg| match arg.as_ref() {
        Argument::OutputFile(path_buf) => Some(path_buf.as_path()),
        _ => None,
    })
    .collect_vec();*/
    todo!("handle output files");

    let stderr = BufReader::new(child.stderr.take().unwrap());
    let stdout = BufReader::new(child.stdout.take().unwrap());
    let stdin = child.stdin.take().unwrap();
    let (txerr, txout) = (tx.clone(), tx.clone());

    let stdout = tokio::spawn(async move { process_stdout(stdout, txout).await });
    let stderr = tokio::spawn(async move { process_stderr(stderr, txerr).await });

    //for line in child.

    Ok(Handle {
        stderr: stderr,
        stdout: stdout,
        channel: rx,
    })
}

async fn process_stdout(
    stream: BufReader<ChildStdout>,
    tx: UnboundedSender<EncodeStatusUpdate>,
) -> Result<()> {
    let rgx_frame = Regex::new(r"frame=(\d+)")?;
    let rgx_fps = Regex::new(r"fps=(\d+(\.\d+)?)")?;
    let rgx_size = Regex::new(r"size=\s*(\d+)KiB")?;
    let rgx_time = Regex::new(r"time=(\d+):(\d+):(\d+).(\d+)")?;
    let rgx_bitrate = Regex::new(r"bitrate=(\d+(\.\d+)?)kbit/s")?;
    let rgx_speed = Regex::new(r"speed=(\d+(\.\d+)?)x")?;

    let mut lines = stream.lines();
    while let Some(line) = lines.next_line().await? {
        let mut response = EncodeProgress::default();
        if let Some(frame) = rgx_frame.captures(&line) {
            let (_, [frame]) = frame.extract();
            match frame.parse() {
                Ok(frame) => response.frame = Some(frame),
                Err(e) => error!(line, frame, "{e:#}"),
            }
        }
        if let Some(fps) = rgx_fps.captures(&line) {
            let (_, [fps]) = fps.extract();
            match fps.parse() {
                Ok(fps) => response.fps = Some(fps),
                Err(e) => error!(line, fps, "{e:#}"),
            }
        }
        if let Some(size) = rgx_size.captures(&line) {
            let (_, [size]) = size.extract();
            match size.parse::<usize>() {
                Ok(size) => response.size = Some(Size::from_kib(size)),
                Err(e) => error!(line, size, "{e:#}"),
            }
        }
        if let Some(time) = rgx_time.captures(&line) {
            let (_, time) = time.extract::<4>();
            let x: Result<Vec<u16>> = time
                .iter()
                .map(|t| str::parse::<u16>(t).map_err(color_eyre::Report::from))
                .collect();
            let [h, m, s, ms] = x?.as_slice() else {
                bail!("something wrong with the parsing logic\n{line}");
            };
        }

        if let Some(fps) = rgx_fps.captures(&line) {
            let (_, [fps]) = fps.extract();
            match fps.parse() {
                Ok(fps) => response.fps = Some(fps),
                Err(e) => error!(line, fps, "{e:#}"),
            }
        }
        if let Some(fps) = rgx_fps.captures(&line) {
            let (_, [fps]) = fps.extract();
            match fps.parse() {
                Ok(fps) => response.fps = Some(fps),
                Err(e) => error!(line, fps, "{e:#}"),
            }
        }

        tx.send(EncodeStatusUpdate::Progress(response))?;
    }

    todo!()
}

async fn process_stderr(
    stream: BufReader<ChildStderr>,
    tx: UnboundedSender<EncodeStatusUpdate>,
) -> Result<()> {
    todo!()
}
