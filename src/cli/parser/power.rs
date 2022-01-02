use std::str::FromStr;

use measurements::Power;

use crate::cli::parser::number::Float;
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub(super) struct Watts(Power);

impl FromStr for Watts {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let v = f64::parse(s)?;
        let v = Power::from_watts(v);
        let s = Self(v);
        Ok(s)
    }
}

impl From<Watts> for Power {
    fn from(v: Watts) -> Self {
        v.0
    }
}
