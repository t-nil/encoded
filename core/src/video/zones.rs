use std::rc::Rc;

use color_eyre::Result;
use derive_more::derive::{Deref, Into};
use itertools::Itertools;
use regex::Regex;
use tracing::warn;

use crate::util::duration_interval::NonOverlappingSortedDurationIntervals;

use super::encoder::params::EncodeSettings;

#[derive(Debug, Clone)]
pub struct InputStats {
    zones: Zones,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Zone {
    name: String,
    settings: Rc<EncodeSettings>,
}

// FIXME also needs to be non-overlapping, so abstract `Pauses` into `NonOverlappingSortedVecOfTimeIntervals` or such :D
#[derive(Debug, Clone, Deref, Into)]
pub struct Zones(NonOverlappingSortedDurationIntervals<Zone>);

impl Zones {
    // TODO tests
    pub fn from_ogm(ogm: &str) -> Result<()> {
        let rgx = Regex::new(r#"CHAPTER(\d{1,2})(NAME)?=(.*)"#).expect("static regex failed");

        let name_buffer = Vec::<Option<Zone>>::new();

        ogm.lines()
            .filter_map(|line| {
                let Some(groups) = rgx.captures(line) else {
                    warn!("parsing OGM: garbage line: {line}");
                    return Option::<()>::None;
                };
                match groups.iter().collect_vec().as_slice() {
                    [chapter_n, timestamp] => {}
                    [chapter_n, _, name] => {}

                    _ => unreachable!(),
                }
                todo!()
            })
            .collect_vec();

        todo!()
    }
}
