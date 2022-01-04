use std::str::FromStr;

use crate::{Error, Result};

pub(crate) trait Integer: num::Integer + Copy + FromStr {
    fn parse(s: &str) -> Result<Self> {
        s.parse::<Self>()
            .map_err(|_| Error::parse_value(format!("could not parse as integer: {}", s)))
    }
}

impl Integer for u64 {}

pub(crate) trait Float: num::Float + Copy + FromStr {
    fn parse(s: &str) -> Result<Self> {
        s.parse::<Self>()
            .map_err(|_| Error::parse_value(format!("could not parse as float: {}", s)))
    }
}

impl Float for f64 {}
