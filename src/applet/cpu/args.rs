use crate::cli::{Arg, Parser};
use crate::Result;

const CPU: &str = "cpu";
const ON: &str = "on";
const GOV: &str = "gov";
const MIN: &str = "min";
const MAX: &str = "max";
const EPB: &str = "epb";
const EPP: &str = "epp";
const QUIET: &str = "quiet";

const CPU_SHORT: char = 'c';
const ON_SHORT: char = 'o';
const GOV_SHORT: char = 'g';
const MIN_SHORT: char = 'n';
const MAX_SHORT: char = 'x';
const EPB_SHORT: char = 'b';
const EPP_SHORT: char = 'p';
const QUIET_SHORT: char = 'q';

const CPU_HELP: &str = "Target cpu ids";
const ON_HELP: &str = "Set cpu online or offline";
const GOV_HELP: &str = "Set cpu governor";
const MIN_HELP: &str = "Set cpu min freq in megahertz";
const MAX_HELP: &str = "Set cpu max freq in megahertz";
const EPB_HELP: &str = "Set cpu epb";
const EPP_HELP: &str = "Set cpu epp";
const QUIET_HELP: &str = "Do not print tables";

#[rustfmt::skip]
fn cpu_help_long() -> String {
"Target cpu ids as comma-delimited list
of integers and/or inclusive ranges
Range syntax: X..Y X.. ..Y ..

".to_string()
}

#[rustfmt::skip]
fn on_help_long() -> String {
    format!(
"Set cpu online or offline per -{}/--{}
Bool syntax: 0 1 true false",
CPU_SHORT, CPU)
}

fn gov_help_long() -> String {
    format!("Set cpu governor per -{}/--{}", CPU_SHORT, CPU)
}

fn min_help_long() -> String {
    format!("Set cpu min freq in megahertz per -{}/--{}", CPU_SHORT, CPU)
}

fn max_help_long() -> String {
    format!("Set cpu max freq in megahertz per -{}/--{}", CPU_SHORT, CPU)
}

#[rustfmt::skip]
fn epb_help_long() -> String {
    format!("Set cpu pstate energy/performance bias per -{}/--{}", CPU_SHORT, CPU)
}

fn epp_help_long() -> String {
    format!(
        "Set cpu pstate energy/performance preference per -{}/--{}",
        CPU_SHORT, CPU
    )
}

pub(super) fn args() -> Vec<Arg> {
    vec![
        Arg {
            name: CPU.into(),
            long: CPU.into(),
            short: CPU_SHORT.into(),
            value_name: "IDS".into(),
            help: CPU_HELP.into(),
            help_long: cpu_help_long().into(),
            ..Default::default()
        },
        Arg {
            name: ON.into(),
            long: ON.into(),
            short: ON_SHORT.into(),
            value_name: "BOOL".into(),
            help: ON_HELP.into(),
            help_long: on_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: GOV.into(),
            long: GOV.into(),
            short: GOV_SHORT.into(),
            value_name: "STR".into(),
            help: GOV_HELP.into(),
            help_long: gov_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: MIN.into(),
            long: MIN.into(),
            short: MIN_SHORT.into(),
            value_name: "INT".into(),
            help: MIN_HELP.into(),
            help_long: min_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: MAX.into(),
            long: MAX.into(),
            short: MAX_SHORT.into(),
            value_name: "INT".into(),
            help: MAX_HELP.into(),
            help_long: max_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: EPB.into(),
            long: EPB.into(),
            short: EPB_SHORT.into(),
            value_name: "INT".into(),
            help: EPB_HELP.into(),
            help_long: epb_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: EPP.into(),
            long: EPP.into(),
            short: EPP_SHORT.into(),
            value_name: "STR".into(),
            help: EPP_HELP.into(),
            help_long: epp_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: QUIET.into(),
            long: QUIET.into(),
            short: QUIET_SHORT.into(),
            help: QUIET_HELP.into(),
            ..Default::default()
        },
    ]
}

impl super::Values {
    pub(super) async fn from_parser(p: Parser<'_>) -> Result<Self> {
        log::trace!("cpu parse start");
        let r = Self {
            ids: p.cpu_ids(CPU).await?,
            on: p.bool(ON)?,
            gov: p.string(GOV),
            min: p.megahertz(MIN)?,
            max: p.megahertz(MAX)?,
            epb: p.int(EPB)?,
            epp: p.string(EPP),
            quiet: p.flag(QUIET),
        };
        log::trace!("cpu parse done");
        Ok(r)
    }
}
