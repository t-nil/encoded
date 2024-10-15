pub mod params;

use std::{path::Path, process::ExitStatus, rc::Rc};

use chrono::{DateTime, Duration, Utc};
use percent_rs::Percentage;
use strum::Display;
use thiserror::Error;

pub trait Encoder {
    fn status(&self) -> EncodingStatus;

    fn start(&mut self, path: &Path) -> Result<(), EncodingError>;
    fn pause(&mut self) -> Result<(), PauseError>;
    fn resume(&mut self) -> Result<(), ResumeError>;
}

#[derive(Debug, Error, Display)]
pub enum PauseError {
    NotStarted,
    AlreadyPaused,
}

#[derive(Debug, Error, Display)]
pub enum ResumeError {
    NotStarted,
    AlreadyRunning,
}

enum EncodingStatus {
    WaitingForStart,
    Running(Box<dyn EncodingProgress>),
    Completed,
    Failed(EncodingError),
    Paused,
}

trait EncodingProgress {
    fn params(&self) -> Rc<dyn Encoder>;

    fn percentage(&self) -> Option<Percentage>;
    fn current_frames(&self) -> Option<usize>;
    fn total_frames(&self) -> Option<usize>;
    fn eta(&self) -> Option<Duration>;

    fn elapsed(&self) -> Duration;
    fn started(&self) -> DateTime<Utc>;
}

struct EncodingProgressImpl {
    started: DateTime<Utc>,
    elapsed: Duration,

    params: Rc<dyn Encoder>,

    percentage: Option<Percentage>,
    current_frames: Option<usize>,
    total_frames: Option<usize>,
    eta: Option<Duration>,
}

impl EncodingProgress for EncodingProgressImpl {
    fn params(&self) -> Rc<dyn Encoder> {
        self.params.clone()
    }

    fn percentage(&self) -> Option<Percentage> {
        self.percentage
    }

    fn current_frames(&self) -> Option<usize> {
        self.current_frames
    }

    fn total_frames(&self) -> Option<usize> {
        self.total_frames
    }

    fn eta(&self) -> Option<Duration> {
        self.eta
    }

    fn elapsed(&self) -> Duration {
        self.elapsed
    }

    fn started(&self) -> DateTime<Utc> {
        self.started
    }
}

#[derive(Debug, Error)]
enum EncodingError {
    #[error("target file already exists")]
    TargetExists,
    #[error("encoder failed with status {status}:\n{stderr}")]
    EncoderFailed { status: ExitStatus, stderr: String },
    #[error("Other: {0}")]
    Other(String),
}

mod svt_av1 {
    use std::path::Path;

    use super::{Encoder, EncodingError, EncodingStatus, PauseError, ResumeError};

    pub struct SvtAV1 {
        params: Vec<Box<dyn Param>>,
    }

    pub trait Param: super::params::Param {}

    impl Encoder for SvtAV1 {
        fn status(&self) -> EncodingStatus {
            todo!()
        }

        fn start(&mut self, path: &Path) -> Result<(), EncodingError> {
            todo!()
        }

        fn pause(&mut self) -> Result<(), PauseError> {
            todo!()
        }

        fn resume(&mut self) -> Result<(), ResumeError> {
            todo!()
        }
    }

    pub mod params {
        use crate::video::encoder::params::param;

        use std::{collections::HashMap, sync::LazyLock};

        use color_eyre::eyre::ensure;
        use derive_builder::Builder;
        use tracing::warn;

        use super::Param;

        //init_param_map! {}

        param!(Crf, n: u8, /* greetz to ramon */ 30, {
            ensure!(n <= 51);
        });

        param!(
            Preset,
            p: u8,
            5u8, /* works on my pc */
            {
                ensure!(p <= 13);
                if p == 13 {
                    warn!("SVT-AV1: Using preset 13 which is only meant for specific use cases and not fit for general encoding.")
                }
                ensure!(p != 6, "Preset 6 is no more! (merged into 7)")
            }
        );

        param!(FilmGrain, n: u8, 7u8, {
            ensure!(n <= 50);
        });

        param!(FilmGrainDenoise, on: u8, 1u8, {
            ensure!(on <= 1);
        });

        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
        pub enum Tune {
            VisualQuality = 0,
            PSNR = 1,
            SSIM = 2,
        }

        // TODO refactor Param so that key() takes &self, and key() and val() return arrays. Then we can implement structs/enums which generate ranges of params (e.g. var. boost params make no sense without it being enabled)
        param!(EnableVarianceBoost, on: u8, 1u8, {
            ensure!(on <= 1);
        });

        param!(VarianceBoostStrength, n: u8, 2u8, {
            ensure!(1<= n && n <= 4);
        });

        param!(VarianceBoostOctile, n: u8, 6u8, {
            ensure!(1 <= n && n <= 8);
        });
    }
}

mod svt_av1_psy {
    use color_eyre::eyre::ensure;

    use super::params::param;

    pub struct SvtAV1Psy {
        params: Vec<Box<dyn Param>>,
    }

    pub trait Param: super::params::Param {}
    impl<T> Param for T where T: super::svt_av1::Param {}

    param!(Sharpness, n: i8, 0i8, {
        ensure!(-7 <= n && n <= 7);
    });

    param!(TfStrength, n: u8, 0u8, { // default is 1, and I haven't tested it, but person on Discord said it's not that good and setting to 0 allows swapping out the grain file
        ensure!(n <= 3);
    });

    param!(FrameLumaBias, n: u8, 0u8, {
        ensure!( n <= 100);
    });

    param!(QpScaleCompressStrength, n: u8, 1u8, {
        ensure!(n <= 3);
    });
}
