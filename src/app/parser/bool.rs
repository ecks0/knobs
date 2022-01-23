use std::str::FromStr;

use crate::{Error, Result};

#[derive(Debug)]
pub(super) struct Bool(bool);

impl FromStr for Bool {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.to_lowercase();
        match s.as_str() {
            "0" | "false" => Ok(Self(false)),
            "1" | "true" => Ok(Self(true)),
            _ => Err(Error::parse_value(format!(
                "expected 0, 1, false, or true, got {:?}",
                s
            ))),
        }
    }
}

impl From<Bool> for bool {
    fn from(v: Bool) -> Self {
        v.0
    }
}
