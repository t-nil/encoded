use std::{path::Path, time::UNIX_EPOCH};

use chrono::{DateTime, Duration, Local, NaiveDateTime};
use color_eyre::{eyre::eyre as anyhow, Result};
use tracing::instrument;

const FMT_GAME_RECORDING_DATE: &str = "%Y-%m-%d_%H-%M-%S";
const GAME_RECORDING_CONCAT_THRESHOLD: Duration = Duration::seconds(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Concat {
    Yes,
    No,
    Unsure,
}

#[instrument(fields(
    prev = prev.as_ref().to_string_lossy().into_owned(),
    next = next.as_ref().to_string_lossy().into_owned(),
    prev_mtime, next_start, time_delta
), ret)]
pub fn should_concat(prev: impl AsRef<Path>, next: impl AsRef<Path>) -> Result<Concat> {
    let (prev, next) = (prev.as_ref(), next.as_ref());
    let prev_mtime = prev.metadata()?.modified()?;
    let prev_mtime: DateTime<Local> = prev_mtime.into();
    let prev_mtime = prev_mtime.naive_local();
    let next_start = NaiveDateTime::parse_and_remainder(
        next.file_name()
            .ok_or_else(|| anyhow!("path has no filename: next=`{next:?}`"))?
            .to_str()
            .ok_or_else(|| anyhow!("not valid UTF8: {}", next.to_string_lossy()))?,
        FMT_GAME_RECORDING_DATE,
    )?
    .0;

    let time_delta = next_start - prev_mtime;

    // logging
    {
        tracing::Span::current().record("prev_mtime", &prev_mtime.to_string());
        tracing::Span::current().record("next_start", &next_start.to_string());
        tracing::Span::current().record("time_delta", &time_delta.to_string());
    }

    let prev_start = NaiveDateTime::parse_and_remainder(
        prev.file_name()
            .ok_or_else(|| anyhow!("path has no filename: next=`{next:?}`"))?
            .to_str()
            .ok_or_else(|| anyhow!("not valid UTF8: {}", next.to_string_lossy()))?,
        FMT_GAME_RECORDING_DATE,
    )?
    .0;
    Ok(if prev_mtime < prev_start {
        Concat::Unsure
    } else {
        match time_delta {
            d if d > GAME_RECORDING_CONCAT_THRESHOLD => Concat::No,
            d if (-GAME_RECORDING_CONCAT_THRESHOLD <= d
                && d <= GAME_RECORDING_CONCAT_THRESHOLD) =>
            {
                Concat::Yes
            }
            d if d <= -GAME_RECORDING_CONCAT_THRESHOLD => Concat::Unsure,
            _ => unreachable!(),
        }
    })
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use std::{env, fs::File, process::Command};

    use chrono::{DateTime, Duration, Local, NaiveDateTime};
    use itertools::Itertools as _;
    use tempfile::tempdir;
    use test_log::test;

    use super::*;

    #[test]
    fn time_fmt_specifier() -> Result<()> {
        assert_eq!(
            chrono::NaiveDate::from_ymd_opt(2024, 9, 1)
                .expect("could not create static date")
                .and_hms_opt(9, 7, 6)
                .expect("could not create static time")
                .format(FMT_GAME_RECORDING_DATE)
                .to_string(),
            "2024-09-01_09-07-06"
        );
        Ok(())
    }

    fn should_concat__helper(files: &[&str], offsets: &[Duration]) -> Result<Vec<Concat>> {
        let tmpdir = tempdir()?;
        let files = files
            .into_iter()
            .map(|p| tmpdir.path().join(p))
            .collect_vec();

        let prev = &files[0];
        let _prev_date = NaiveDateTime::parse_and_remainder(
            &prev.file_name().unwrap().to_str().unwrap(),
            FMT_GAME_RECORDING_DATE,
        )?
        .0
        .and_local_timezone(Local)
        .unwrap();
        files[1..]
            .iter()
            .flat_map(|next| -> Result<Vec<Result<_>>> {
                let next_date = NaiveDateTime::parse_and_remainder(
                    next.file_name().unwrap().to_str().unwrap(),
                    FMT_GAME_RECORDING_DATE,
                )?
                .0
                .and_local_timezone(Local)
                .unwrap();
                Ok(offsets
                    .iter()
                    .copied()
                    .map(|offset| {
                        let next_date = next_date + offset;
                        {
                            let f = File::create(Path::new(&prev))?;
                            f.set_modified(next_date.into())?;
                            f.sync_all()?;
                        };
                        should_concat(&prev, next)
                    })
                    .collect_vec())
            })
            .flatten()
            .collect()
    }

    static SHOULD_CONCAT_FILES: [&str; 7] = [
        "2024-09-30_19-17-16 [Hogwarts Legacy].mkv",
        "2024-09-30_19-27-14 [Hogwarts Legacy].mkv",
        "2024-09-30_19-27-16 [Hogwarts Legacy].mkv",
        "2024-09-30_19-27-17 [Hogwarts Legacy].mkv",
        "2024-09-30_19-27-22 [Hogwarts Legacy].mkv",
        "2024-09-30_19-34-49 [Hogwarts Legacy].mkv",
        "2024-09-30_20-11-35 [Hogwarts Legacy].mkv",
    ];

    #[test]
    fn should_concat__yes() -> Result<()> {
        let offsets = [-5, -2, 1, 3, 5]
            .into_iter()
            .map(|sec| Duration::seconds(sec))
            .collect_vec();
        assert!(should_concat__helper(&SHOULD_CONCAT_FILES, &offsets)?
            .into_iter()
            .all(|result| result == Concat::Yes));
        Ok(())
    }
    #[test]
    fn should_concat__no() -> Result<()> {
        // extra file list, because mtime before ctime (parsed from title), which we would have with the 10min intervals, would be an Unsure case
        let files = &[
            "2024-09-27_19-17-16 [Hogwarts Legacy].mkv",
            "2024-09-30_20-11-35 [Hogwarts Legacy].mkv",
        ];
        let offsets = [-10, -500, -4000, -3600 * 47]
            .into_iter()
            .map(|sec| Duration::seconds(sec))
            .collect_vec();
        assert!(should_concat__helper(files, &offsets)?
            .into_iter()
            .all(|result| result == Concat::No));
        Ok(())
    }
    #[test]
    fn should_concat__unsure() -> Result<()> {
        let offsets = [10, 300, 3600 * 24 + 5000, -3600 * 48] // last value has to be bigger than the largest difference between (prev, next) in the test set, because it triggers `Unsure` on mtime<ctime.
            .into_iter()
            .map(|sec| Duration::seconds(sec))
            .collect_vec();
        assert!(should_concat__helper(&SHOULD_CONCAT_FILES, &offsets)?
            .into_iter()
            .all(|result| result == Concat::Unsure));
        Ok(())
    }
}
