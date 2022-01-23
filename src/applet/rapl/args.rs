use crate::cli::{Arg, Parser};
use crate::Result;

const PACKAGE: &str = "package";
const SUBZONE: &str = "subzone";
const CONSTRAINT: &str = "constraint";
const LIMIT: &str = "limit";
const WINDOW: &str = "window";

const PACKAGE_SHORT: char = 'p';
const SUBZONE_SHORT: char = 's';
const CONSTRAINT_SHORT: char = 'c';
const LIMIT_SHORT: char = 'l';
const WINDOW_SHORT: char = 'w';

const PACKAGE_HELP: &str = "Target rapl package";
const SUBZONE_HELP: &str = "Target rapl subzone";
const CONSTRAINT_HELP: &str = "Target rapl constraint";
const LIMIT_HELP: &str = "Set rapl power limit in watts";
const WINDOW_HELP: &str = "Set rapl power window in microseconds";

#[rustfmt::skip]
fn limit_help_long() -> String {
    format!(
"Set rapl power limit in watts per
--{}/{}/{}",
    PACKAGE, SUBZONE, CONSTRAINT)
}

#[rustfmt::skip]
fn window_help_long() -> String {
    format!(
"Set rapl power window in microseconds per
--{}/{}/{}",
    PACKAGE, SUBZONE, CONSTRAINT)
}

pub(super) fn args() -> Vec<Arg> {
    vec![
        Arg {
            name: PACKAGE.into(),
            long: PACKAGE.into(),
            short: PACKAGE_SHORT.into(),
            value_name: "INT".into(),
            help: PACKAGE_HELP.into(),
            ..Default::default()
        },
        Arg {
            name: SUBZONE.into(),
            long: SUBZONE.into(),
            short: SUBZONE_SHORT.into(),
            value_name: "INT".into(),
            help: SUBZONE_HELP.into(),
            ..Default::default()
        },
        Arg {
            name: CONSTRAINT.into(),
            long: CONSTRAINT.into(),
            short: CONSTRAINT_SHORT.into(),
            value_name: "INT".into(),
            help: CONSTRAINT_HELP.into(),
            ..Default::default()
        },
        Arg {
            name: LIMIT.into(),
            long: LIMIT.into(),
            short: LIMIT_SHORT.into(),
            value_name: "FLOAT".into(),
            help: LIMIT_HELP.into(),
            help_long: limit_help_long().into(),
            requires: vec![PACKAGE, CONSTRAINT].into(),
            ..Default::default()
        },
        Arg {
            name: WINDOW.into(),
            long: WINDOW.into(),
            short: WINDOW_SHORT.into(),
            value_name: "INT".into(),
            help: WINDOW_HELP.into(),
            help_long: window_help_long().into(),
            requires: vec![PACKAGE, CONSTRAINT].into(),
            ..Default::default()
        },
    ]
}

impl super::Values {
    pub(super) async fn from_parser(p: Parser<'_>) -> Result<Self> {
        log::trace!("rapl parse start");
        let r = Self {
            constraint_ids: p.rapl_constraint_ids(PACKAGE, SUBZONE, CONSTRAINT).await?,
            limit: p.watts(LIMIT)?,
            window: p.microseconds(WINDOW)?,
        };
        log::trace!("rapl parse done");
        Ok(r)
    }
}
