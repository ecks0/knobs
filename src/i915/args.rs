use async_trait::async_trait;

use crate::cli::{Arg, I915Driver, Parser};
use crate::util::convert::TryFromRef;
use crate::{Error, Result};

const I915: &str = "i915";
const I915_MIN: &str = "i915-min";
const I915_MAX: &str = "i915-max";
const I915_BOOST: &str = "i915-boost";

#[async_trait]
impl<'a> TryFromRef<Parser<'a>> for super::I915 {
    type Error = Error;

    async fn try_from_ref(p: &Parser<'a>) -> Result<Self> {
        let r = Self {
            i915: p.drm_ids::<I915Driver>(I915).await?,
            i915_min: p.megahertz(I915_MIN)?,
            i915_max: p.megahertz(I915_MAX)?,
            i915_boost: p.megahertz(I915_BOOST)?,
        };
        Ok(r)
    }
}

pub(super) fn args() -> impl IntoIterator<Item = Arg> {
    vec![
        Arg {
            name: I915,
            long: I915.into(),
            short: None,
            value_name: "IDS".into(),
            help: i915_help().into(),
            help_long: i915_help_long().into(),
            requires: None,
            conflicts: None,
        },
        Arg {
            name: I915_MIN,
            long: I915_MIN.into(),
            short: None,
            value_name: "MHZ".into(),
            help: i915_min_help().into(),
            help_long: i915_min_help_long().into(),
            requires: vec![I915].into(),
            conflicts: None,
        },
        Arg {
            name: I915_MAX,
            long: I915_MAX.into(),
            short: None,
            value_name: "MHZ".into(),
            help: i915_max_help().into(),
            help_long: i915_max_help_long().into(),
            requires: vec![I915].into(),
            conflicts: None,
        },
        Arg {
            name: I915_BOOST,
            long: I915_BOOST.into(),
            short: None,
            value_name: "MHZ".into(),
            help: i915_boost_help().into(),
            help_long: i915_boost_help_long().into(),
            requires: vec![I915].into(),
            conflicts: None,
        },
    ]
}

fn i915_help() -> String {
    "Target i915 drm ids or bus ids".to_string()
}

fn i915_help_long() -> String {
    "Target i915 drm ids or bus ids, comma-delimited. Bus id syntax e.g. pci:0000:00:02.0"
        .to_string()
}

fn i915_min_help() -> String {
    "Set i915 min freq in megahertz".to_string()
}

fn i915_min_help_long() -> String {
    format!("Set i915 min freq in megahertz per --{}", I915)
}

fn i915_max_help() -> String {
    "Set i915 max freq in megahertz".to_string()
}

fn i915_max_help_long() -> String {
    format!("Set i915 max freq in megahertz per --{}", I915)
}

fn i915_boost_help() -> String {
    "Set i915 boost freq in megahertz".to_string()
}

fn i915_boost_help_long() -> String {
    format!("Set i915 boost freq in megahertz per --{}", I915)
}
