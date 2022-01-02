use std::str::FromStr;

use crate::{Error, Result};

pub(crate) trait Integer: num::Integer + Copy + FromStr {
    fn parse(s: &str) -> Result<Self> {
        s.parse::<Self>().map_err(|_| Error::parse_value("Could not parse string as integer"))
    }
}

impl Integer for u64 {}

pub(crate) trait Float: num::Float + Copy + FromStr {
    fn parse(s: &str) -> Result<Self> {
        s.parse::<Self>().map_err(|_| Error::parse_value("Could not parse string as float"))
    }
}

impl Float for f64 {}
