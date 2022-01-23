use crate::app::{Arg, I915Driver, Parser};
use crate::Result;

const CARD: &str = "card";
const MIN: &str = "min";
const MAX: &str = "max";
const BOOST: &str = "boost";

const CARD_SHORT: char = 'c';
const MIN_SHORT: char = 'n';
const MAX_SHORT: char = 'x';
const BOOST_SHORT: char = 'b';

const CARD_HELP: &str = "Target i915 drm card indexes or bus ids";
const MIN_HELP: &str = "Set i915 min freq in megahertz";
const MAX_HELP: &str = "Set i915 max freq in megahertz";
const BOOST_HELP: &str = "Set i915 boost freq in megahertz";

#[rustfmt::skip]
fn card_help_long() -> String {
"Target i915 drm card indexes or bus ids, comma-delimited
Bus id syntax: BUS:ID e.g. pci:0000:00:02.0".to_string()
}

fn min_help_long() -> String {
    format!("Set i915 min freq in megahertz per --{}", CARD)
}

fn max_help_long() -> String {
    format!("Set i915 max freq in megahertz per --{}", CARD)
}

fn boost_help_long() -> String {
    format!("Set i915 boost freq in megahertz per --{}", CARD)
}

pub(super) fn args() -> Vec<Arg> {
    vec![
        Arg {
            name: CARD.into(),
            long: CARD.into(),
            short: CARD_SHORT.into(),
            value_name: "IDS".into(),
            help: CARD_HELP.into(),
            help_long: card_help_long().into(),
            ..Default::default()
        },
        Arg {
            name: MIN.into(),
            long: MIN.into(),
            short: MIN_SHORT.into(),
            value_name: "INT".into(),
            help: MIN_HELP.into(),
            help_long: min_help_long().into(),
            requires: vec![CARD].into(),
            ..Default::default()
        },
        Arg {
            name: MAX.into(),
            long: MAX.into(),
            short: MAX_SHORT.into(),
            value_name: "INT".into(),
            help: MAX_HELP.into(),
            help_long: max_help_long().into(),
            requires: vec![CARD].into(),
            ..Default::default()
        },
        Arg {
            name: BOOST.into(),
            long: BOOST.into(),
            short: BOOST_SHORT.into(),
            value_name: "INT".into(),
            help: BOOST_HELP.into(),
            help_long: boost_help_long().into(),
            requires: vec![CARD].into(),
            ..Default::default()
        },
    ]
}

impl super::Values {
    pub(super) async fn from_parser(p: Parser<'_>) -> Result<Self> {
        log::trace!("i915 parse start");
        let r = Self {
            ids: p.drm_ids::<I915Driver>(CARD).await?,
            min: p.megahertz(MIN)?,
            max: p.megahertz(MAX)?,
            boost: p.megahertz(BOOST)?,
        };
        log::trace!("i915 parse done");
        Ok(r)
    }
}
