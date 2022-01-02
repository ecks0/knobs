use std::str::FromStr;
use std::time::Duration;

use crate::cli::parser::number::Integer;
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub(super) struct Microseconds(Duration);

impl FromStr for Microseconds {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let v = u64::parse(s)?;
        let v = Duration::from_micros(v);
        let s = Self(v);
        Ok(s)
    }
}

impl From<Microseconds> for Duration {
    fn from(v: Microseconds) -> Self {
        v.0
    }
}
