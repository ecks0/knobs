use async_trait::async_trait;

use crate::cli::{Arg, I915Driver, Parser};
use crate::util::convert::TryFromRef;
use crate::{Error, Result};

const I915: &str = "i915";
const I915_MIN: &str = "i915-min";
const I915_MAX: &str = "i915-max";
const I915_BOOST: &str = "i915-boost";

#[async_trait]
impl TryFromRef<Parser> for super::I915 {
    type Error = Error;

    async fn try_from_ref(p: &Parser) -> Result<Self> {
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
            name: I915.into(),
            long: I915.into(),
            value_name: "IDS".into(),
            help: i915_help().into(),
            help_long: i915_help_long().into(),
            ..Default::default()
        },
        Arg {
            name: I915_MIN.into(),
            long: I915_MIN.into(),
            value_name: "INT".into(),
            help: i915_min_help().into(),
            help_long: i915_min_help_long().into(),
            requires: vec![I915].into(),
            ..Default::default()
        },
        Arg {
            name: I915_MAX.into(),
            long: I915_MAX.into(),
            value_name: "INT".into(),
            help: i915_max_help().into(),
            help_long: i915_max_help_long().into(),
            requires: vec![I915].into(),
            ..Default::default()
        },
        Arg {
            name: I915_BOOST.into(),
            long: I915_BOOST.into(),
            value_name: "INT".into(),
            help: i915_boost_help().into(),
            help_long: i915_boost_help_long().into(),
            requires: vec![I915].into(),
            ..Default::default()
        },
    ]
}

fn i915_help() -> String {
    "Target i915 drm card indexes or bus ids".to_string()
}

#[rustfmt::skip]
fn i915_help_long() -> String {
"Target i915 drm card indexes or bus ids, comma-delimited
Bus id syntax: BUS:ID e.g. pci:0000:00:02.0
".to_string()
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
