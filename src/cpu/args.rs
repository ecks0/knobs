use async_trait::async_trait;

use crate::cli::{Arg, Parser};
use crate::util::convert::TryFromRef;
use crate::{Error, Result};

const CPU: &str = "cpu";
const CPU_ON: &str = "cpu-on";
const CPU_GOV: &str = "cpu-gov";
const CPU_MIN: &str = "cpu-min";
const CPU_MAX: &str = "cpu-max";
const CPU_EPB: &str = "cpu-epb";
const CPU_EPP: &str = "cpu-epp";

const CPU_SHORT: &str = "c";
const CPU_ON_SHORT: &str = "o";
const CPU_GOV_SHORT: &str = "g";
const CPU_MIN_SHORT: &str = "n";
const CPU_MAX_SHORT: &str = "x";

#[async_trait]
impl<'a> TryFromRef<Parser<'a>> for super::Cpu {
    type Error = Error;

    async fn try_from_ref(p: &Parser<'a>) -> Result<Self> {
        let r = Self {
            cpu: p.cpu_ids(CPU).await?,
            cpu_on: p.bool(CPU_ON)?,
            cpu_gov: p.string(CPU_GOV),
            cpu_min: p.megahertz(CPU_MIN)?,
            cpu_max: p.megahertz(CPU_MAX)?,
            cpu_epb: p.pstate_epb(CPU_EPB)?,
            cpu_epp: p.string(CPU_EPP),
        };
        Ok(r)
    }
}

pub(super) fn args() -> impl IntoIterator<Item = Arg> {
    vec![
        Arg {
            name: CPU.into(),
            long: CPU.into(),
            short: CPU_SHORT.into(),
            value_name: "IDS".into(),
            help: cpu_help().into(),
            help_long: cpu_help_long().into(),
            ..Default::default()
        },
        Arg {
            name: CPU_ON.into(),
            long: CPU_ON.into(),
            short: CPU_ON_SHORT.into(),
            value_name: "BOOL".into(),
            help: cpu_on_help().into(),
            help_long: cpu_on_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: CPU_GOV.into(),
            long: CPU_GOV.into(),
            short: CPU_GOV_SHORT.into(),
            value_name: "STR".into(),
            help: cpu_gov_help().into(),
            help_long: cpu_gov_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: CPU_MIN.into(),
            long: CPU_MIN.into(),
            short: CPU_MIN_SHORT.into(),
            value_name: "INT".into(),
            help: cpu_min_help().into(),
            help_long: cpu_min_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: CPU_MAX.into(),
            long: CPU_MAX.into(),
            short: CPU_MAX_SHORT.into(),
            value_name: "INT".into(),
            help: cpu_max_help().into(),
            help_long: cpu_max_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: CPU_EPB.into(),
            long: CPU_EPB.into(),
            short: None,
            value_name: "0..=15".into(),
            help: cpu_epb_help().into(),
            help_long: cpu_epb_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
        Arg {
            name: CPU_EPP.into(),
            long: CPU_EPP.into(),
            short: None,
            value_name: "STR".into(),
            help: cpu_epp_help().into(),
            help_long: cpu_epp_help_long().into(),
            requires: vec![CPU].into(),
            ..Default::default()
        },
    ]
}

fn cpu_help() -> String {
    "Target cpu ids".to_string()
}

#[rustfmt::skip]
fn cpu_help_long() -> String {
"Target cpu ids as comma-delimited list of
integers and/or inclusive ranges
Range syntax: X..Y X.. ..Y ..

".to_string()
}

fn cpu_on_help() -> String {
    "Set cpu online or offline".to_string()
}

#[rustfmt::skip]
fn cpu_on_help_long() -> String {
    format!(
"Set cpu online or offline per -{}/--{}
Bool syntax: 0 1 true false

", CPU_SHORT, CPU)
}

fn cpu_gov_help() -> String {
    "Set cpu governor".to_string()
}

fn cpu_gov_help_long() -> String {
    format!("Set cpu governor per -{}/--{}", CPU_SHORT, CPU)
}

fn cpu_min_help() -> String {
    "Set cpu min freq in megahertz".to_string()
}

fn cpu_min_help_long() -> String {
    format!("Set cpu min freq in megahertz per -{}/--{}", CPU_SHORT, CPU)
}

fn cpu_max_help() -> String {
    "Set cpu max freq in megahertz".to_string()
}

fn cpu_max_help_long() -> String {
    format!("Set cpu max freq in megahertz per -{}/--{}", CPU_SHORT, CPU)
}

fn cpu_epb_help() -> String {
    "Set cpu epb".to_string()
}

#[rustfmt::skip]
fn cpu_epb_help_long() -> String {
    format!(
"Set cpu pstate energy/performance bias per -{}/--{}.
Expects an integer within 0..=15

", CPU_SHORT, CPU)
}

fn cpu_epp_help() -> String {
    "Set cpu epp".to_string()
}

fn cpu_epp_help_long() -> String {
    format!(
        "Set cpu pstate energy/performance preference per -{}/--{}",
        CPU_SHORT, CPU
    )
}
