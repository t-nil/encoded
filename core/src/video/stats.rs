use chrono::{DateTime, Duration, Utc};
use color_eyre::eyre::{eyre, Report};
use size::Size;

use crate::util::duration_interval::{DurationInterval, NonOverlappingSortedDurationIntervals};

/// Differentiate "encoding stats" (time, resource usage) and "encoded stats" [bitrate, bitrate/time, shares of scene types (still, pause, hq etc)].

pub struct EncodingTime {
    start: DateTime<Utc>,

    time_paused: Vec<DurationInterval>,
}
pub struct EncodedTime {
    duration: DurationInterval,
    time_paused: Vec<DurationInterval>,
}

impl EncodingTime {}

impl EncodedTime {
    pub fn new(
        encoding_time: EncodingTime,
        end: DateTime<Utc>,
    ) -> Result<Self, (Report, EncodingTime)> {
        let EncodingTime { start, time_paused } = &encoding_time;
        if end <= *start {
            return Err((
                eyre!("End must come after start ({start} >= {end})"),
                encoding_time,
            ));
        }

        todo!()
    }
}

pub struct EncodingStats {
    time: EncodingTime,
    cpu_time: Duration,
}

pub struct EncodedStats {
    time: EncodedTime,
    cpu_time: Duration,
    memory_peak: Size,
    memory_avg: Option<Size>,

    bitrate: Size,
    bitrate_over_time: Option<NonOverlappingSortedDurationIntervals<Size>>,
}

impl EncodingStats {
    pub fn new() {}

    /// Total duration of running process(es) subtracted by all pauses.
    pub fn actual_duration(&self) -> Duration {
        todo!()
    }
}
