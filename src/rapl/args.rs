use async_trait::async_trait;

use crate::cli::{Arg, Parser};
use crate::util::convert::TryFromRef;
use crate::{Error, Result};

pub(crate) const RAPL_PACKAGE: &str = "rapl-package";
pub(crate) const RAPL_SUBZONE: &str = "rapl-subzone";
pub(crate) const RAPL_CONSTRAINT: &str = "rapl-constraint";
pub(crate) const RAPL_LIMIT: &str = "rapl-limit";
pub(crate) const RAPL_WINDOW: &str = "rapl-window";

pub(crate) const RAPL_PACKAGE_SHORT: &str = "P";
pub(crate) const RAPL_SUBZONE_SHORT: &str = "S";
pub(crate) const RAPL_CONSTRAINT_SHORT: &str = "C";
pub(crate) const RAPL_LIMIT_SHORT: &str = "L";
pub(crate) const RAPL_WINDOW_SHORT: &str = "W";

#[async_trait]
impl<'a> TryFromRef<Parser<'a>> for super::Rapl {
    type Error = Error;

    async fn try_from_ref(p: &Parser<'a>) -> Result<Self> {
        let r = Self {
            rapl_constraint: p
                .rapl_constraint(RAPL_PACKAGE, RAPL_SUBZONE, RAPL_CONSTRAINT)
                .await?,
            rapl_limit: p.watts(RAPL_LIMIT)?,
            rapl_window: p.microseconds(RAPL_WINDOW)?,
        };
        Ok(r)
    }
}

pub(super) fn args() -> impl IntoIterator<Item = Arg> {
    vec![
        Arg {
            name: RAPL_PACKAGE,
            long: RAPL_PACKAGE.into(),
            short: RAPL_PACKAGE_SHORT.into(),
            value_name: "INT".into(),
            help: rapl_package_help().into(),
            help_long: None,
            requires: None,
            conflicts: None,
        },
        Arg {
            name: RAPL_SUBZONE,
            long: RAPL_SUBZONE.into(),
            short: RAPL_SUBZONE_SHORT.into(),
            value_name: "INT".into(),
            help: rapl_subzone_help().into(),
            help_long: None,
            requires: None,
            conflicts: None,
        },
        Arg {
            name: RAPL_CONSTRAINT,
            long: RAPL_CONSTRAINT.into(),
            short: RAPL_CONSTRAINT_SHORT.into(),
            value_name: "INT".into(),
            help: rapl_constraint_help().into(),
            help_long: None,
            requires: None,
            conflicts: None,
        },
        Arg {
            name: RAPL_LIMIT,
            long: RAPL_LIMIT.into(),
            short: RAPL_LIMIT_SHORT.into(),
            value_name: "WATTS".into(),
            help: rapl_limit_help().into(),
            help_long: rapl_limit_help_long().into(),
            requires: vec![RAPL_PACKAGE, RAPL_CONSTRAINT].into(),
            conflicts: None,
        },
        Arg {
            name: RAPL_WINDOW,
            long: RAPL_WINDOW.into(),
            short: RAPL_WINDOW_SHORT.into(),
            value_name: "μS".into(),
            help: rapl_window_help().into(),
            help_long: rapl_window_help_long().into(),
            requires: vec![RAPL_PACKAGE, RAPL_CONSTRAINT].into(),
            conflicts: None,
        },
    ]
}

pub(crate) fn rapl_package_help() -> String {
    "Target rapl package".to_string()
}

pub(crate) fn rapl_subzone_help() -> String {
    "Target rapl subzone".to_string()
}

pub(crate) fn rapl_constraint_help() -> String {
    "Target rapl constraint".to_string()
}

pub(crate) fn rapl_limit_help() -> String {
    "Set rapl power limit in watts".to_string()
}

pub(crate) fn rapl_limit_help_long() -> String {
    format!(
        "Set rapl power limit in watts per --{}/{}/{}",
        RAPL_PACKAGE, RAPL_SUBZONE, RAPL_CONSTRAINT
    )
}

pub(crate) fn rapl_window_help() -> String {
    "Set rapl power window in microseconds".to_string()
}

pub(crate) fn rapl_window_help_long() -> String {
    format!(
        "Set rapl power window in microseconds per --{}/{}/{}",
        RAPL_PACKAGE, RAPL_SUBZONE, RAPL_CONSTRAINT
    )
}
