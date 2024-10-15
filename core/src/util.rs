use std::ops::Range;

use color_eyre::Result;
use thiserror::Error;

pub mod duration_interval {
    use std::ops::Range;

    use chrono::Duration;
    use color_eyre::Result;
    use derive_more::derive::{Deref, DerefMut, Into};
    use sorted_vec::SortedSet;
    use thiserror::Error;

    /// A time interval that is guaranteed to be valid (start is after end)
    #[derive(Debug, Clone, PartialEq, Eq, Into, Deref, DerefMut)]
    pub struct DurationInterval(Range<Duration>);

    #[derive(Debug, Error, PartialEq, Eq)]
    pub enum DurationIntervalCreateError {
        #[error("End must be after start")]
        StartAfterEnd,
    }

    impl DurationInterval {
        pub fn new(value: Range<Duration>) -> Result<Self, DurationIntervalCreateError> {
            use DurationIntervalCreateError::*;
            if value.start > value.end {
                Err(StartAfterEnd)
            } else {
                Ok(DurationInterval(value))
            }
        }

        pub fn overlaps_with(&self, other: &Self) -> bool {
            self.start < other.end && other.start < self.end
        }
    }
    #[derive(Debug, Clone, Default, Deref, Into)]
    pub struct NonOverlappingSortedDurationIntervals<T>(SortedSet<DurationIntervalMetadata<T>>);

    impl<T> NonOverlappingSortedDurationIntervals<T> {
        pub fn new() -> Self {
            //SortedVec::
            todo!()
        }
    }

    impl TryFrom<Range<Duration>> for DurationInterval {
        type Error = DurationIntervalCreateError;

        fn try_from(value: Range<Duration>) -> Result<Self, Self::Error> {
            Self::new(value)
        }
    }

    impl PartialOrd for DurationInterval {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    // TODO tests!
    impl Ord for DurationInterval {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            use std::cmp::Ordering::*;

            let (this, other) = (&self.0, &other.0);

            if this.start > other.start {
                Greater
            } else if this.start < other.start {
                Less
            } else {
                if this.end > other.end {
                    Greater
                } else if this.end < other.end {
                    Less
                } else {
                    Equal
                }
            }
        }
    }

    #[derive(Debug, Clone, Error)]
    pub enum DurationIntervalError {
        #[error("Pauses overlap: {0:?} and {1:?}")]
        IntervalsOverlap(DurationInterval, DurationInterval),
    }

    #[derive(Debug, Clone, Deref, DerefMut, Into)]
    pub struct DurationIntervalMetadata<T>(pub (DurationInterval, T));

    impl<T> PartialEq for DurationIntervalMetadata<T> {
        fn eq(&self, other: &Self) -> bool {
            self.0 .0 == other.0 .0
        }
    }

    impl<T> Eq for DurationIntervalMetadata<T> {}

    impl<T> Ord for DurationIntervalMetadata<T> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0 .0.cmp(&other.0 .0)
        }
    }

    impl<T> PartialOrd for DurationIntervalMetadata<T> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    /// pauses are guaranteed to be sorted and to not overlap. First is implemented via SortedVec, second via checks in insert()
    impl<T> NonOverlappingSortedDurationIntervals<T> {
        /// inserts only if non-overlapping with every existing pause, otherwise returns the offender inside the error.
        pub fn try_insert(
            &mut self,
            value: DurationIntervalMetadata<T>,
        ) -> Result<(), DurationIntervalError> {
            let (new_interval, _new_meta) = (&value.0 .0, &value.0 .1);
            if let Some(overlap) = self
                .iter()
                .filter(|&interval| interval.0 .0.overlaps_with(new_interval))
                .next()
            {
                return Err(DurationIntervalError::IntervalsOverlap(
                    overlap.0 .0.clone(),
                    new_interval.clone(),
                ));
            }

            self.0.push(value);
            Ok(())
        }

        pub fn extend(
            &mut self,
            intervals: impl Iterator<Item = DurationIntervalMetadata<T>>,
        ) -> Result<()> {
            for interval in intervals {
                self.try_insert(interval)?;
            }
            Ok(())
        }

        /// Some(Duration) => total length of all pauses added
        /// None => pauses were empty
        pub fn total_length(&self) -> Option<chrono::Duration> {
            self.0
                .iter()
                .map(|DurationIntervalMetadata((interval, _))| interval.end - interval.start)
                .reduce(core::ops::Add::add)
        }
    }

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod test {
        use chrono::{Duration, Utc};

        use super::*;

        #[test]
        fn DurationInterval__invariants() {
            let range = Duration::seconds(0)..Duration::seconds(50);
            assert_eq!(
                DurationInterval::new(range.clone()),
                Ok(DurationInterval(range.clone()))
            );
            assert_eq!(
                DurationInterval::try_from(range.clone()),
                Ok(DurationInterval(range.clone()))
            );

            let range = range.end..range.start;
            assert_eq!(
                DurationInterval::new(range.clone()),
                Err(DurationIntervalCreateError::StartAfterEnd)
            );
            assert_eq!(
                DurationInterval::try_from(range.clone()),
                Err(DurationIntervalCreateError::StartAfterEnd)
            );
        }

        #[test]
        fn DurationInterval__overlaps_with() -> Result<()> {
            let now = Duration::seconds(0);
            let (plus30, plus50, plus60) = (
                Duration::seconds(30),
                Duration::seconds(50),
                Duration::seconds(60),
            );
            assert!(DurationInterval::try_from(plus30..plus60)?
                .overlaps_with(&DurationInterval::try_from(now..plus50)?));
            assert!(DurationInterval::try_from(now..plus50)?
                .overlaps_with(&DurationInterval::try_from(plus30..plus60)?));
            assert!(!DurationInterval::try_from(now..plus30)?
                .overlaps_with(&DurationInterval::try_from(plus50..plus60)?));
            assert!(!DurationInterval::try_from(now..plus50)?
                .overlaps_with(&DurationInterval::try_from(plus50..plus60)?));
            assert!(DurationInterval::try_from(now..plus60)?
                .overlaps_with(&DurationInterval::try_from(plus30..plus50)?));

            assert!(!DurationInterval::try_from(now..now)?
                .overlaps_with(&DurationInterval::try_from(now..plus30)?));
            assert!(!DurationInterval::try_from(plus50..plus50)?
                .overlaps_with(&DurationInterval::try_from(plus30..plus50)?));

            Ok(())
        }
    }
}

pub mod bitrate {
    use chrono::Duration;
    use size::Size;

    // TODO (IMPORTANT)
    // TODO (IMPORTANT) tests!!
    pub struct Bitrate {
        bits_per_second: f64,
    }

    impl Bitrate {
        pub fn new(size: Size, time: Duration) -> Self {
            // num_nanoseconds() overflows on >~250 years (i64)
            Self {
                bits_per_second: ((size.bytes() as f64) * 8f64)
                    / ((time
                        .num_nanoseconds()
                        .expect("num_nanoseconds() overflows on >~250 years (i64)")
                        as f64)
                        / (10_usize.pow(9) as f64)),
            }
        }

        // todo maybe macros
        #[allow(non_snake_case)]
        pub fn Kibit_per_s(&self) -> f64 {
            self.bits_per_second / 1024f64
        }

        #[allow(non_snake_case)]
        pub fn Mibit_per_s(&self) -> f64 {
            self.bits_per_second / 1024f64
        }
    }
}
