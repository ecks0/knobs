use std::str::FromStr;

use crate::app::NAME;

pub(crate) fn var_name(name: &str) -> String {
    format!("{}_{}", NAME.to_ascii_uppercase(), name)
}

pub(crate) fn var(name: &str) -> Option<String> {
    std::env::var(&var_name(name)).ok().map(String::from)
}

pub(crate) fn parse<T: FromStr>(name: &str) -> Option<T> {
    var(name).and_then(|v| T::from_str(&v).ok())
}
