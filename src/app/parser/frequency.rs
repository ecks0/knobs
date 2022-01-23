use std::str::FromStr;

use measurements::Frequency;

use crate::app::parser::number::Integer as _;
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub(super) struct Megahertz(Frequency);

impl FromStr for Megahertz {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let i = u64::parse(s)?;
        let f = Frequency::from_megahertz(i as f64);
        let s = Self(f);
        Ok(s)
    }
}

impl From<Megahertz> for Frequency {
    fn from(v: Megahertz) -> Self {
        v.0
    }
}
