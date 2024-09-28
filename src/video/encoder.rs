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
    Completed(EncodedVideo),
    Failed(EncodingError),
    Paused,
}

trait EncodingProgress {
    fn params(&self) -> Rc<dyn Encoder>;

    fn percentage(&self) -> Option<PercentageDecimal>;
    fn current_frames(&self) -> Option<usize>;
    fn total_frames(&self) -> Option<usize>;
    fn eta(&self) -> Option<Duration>;

    fn elapsed(&self) -> Duration;
    fn started(&self) -> Time;
}

struct EncodingProgressImpl {
    started: Time,
    elapsed: Duration,

    params: Rc<dyn Encoder>,

    percentage: Option<PercentageDecimal>,
    current_frames: Option<usize>,
    total_frames: Option<usize>,
    eta: Option<Duration>,
}

pub trait MyClone {
    type Target;

    fn clone(self) -> Target;
    fn clone_from_ref(&self) -> Target;
}

impl MyClone for PercentageDecimal {
    type Target = PercentageDecimal;

    fn clone(self) -> Target {
        Percentage::from_decimal(self.value())
    }

    fn clone_from_ref(&self) -> Target {
        Percentage::from_decimal(self.value())
    }
}

impl EncodingProgress for EncodingProgressImpl {
    fn params(&self) -> Rc<dyn Encoder> {
        self.params.clone()
    }

    fn percentage(&self) -> Option<PercentageDecimal> {
        self.percentage.map(MyClone::clone)
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

    fn started(&self) -> Time {
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

mod av1 {
    use bounded_integer::bounded_integer;

    pub struct AV1 {
        crf: Crf,
        preset: Preset,
    }

    bounded_integer! {
        struct Preset { 0..13 }
    }

    bounded_integer! {struct Crf{0..63}}
}

impl Encoder for AV1 {
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
