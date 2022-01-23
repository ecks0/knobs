use crate::app::{Arg, NvmlDriver, Parser};
use crate::Result;

const CARD: &str = "card";
const GPU_MIN: &str = "gpu-min";
const GPU_MAX: &str = "gpu-max";
const GPU_RESET: &str = "gpu-reset";
const POWER: &str = "power";
const POWER_RESET: &str = "power-reset";

const CARD_SHORT: char = 'c';
const GPU_MIN_SHORT: char = 'n';
const GPU_MAX_SHORT: char = 'x';
const GPU_RESET_SHORT: char = 'r';
const POWER_SHORT: char = 'P';
const POWER_RESET_SHORT: char = 'R';

const CARD_HELP: &str = "Target nvml drm card indexes or bus ids";
const GPU_MIN_HELP: &str = "Set nvml min gpu freq in megahertz";
const GPU_MAX_HELP: &str = "Set nvml max gpu freq in megahertz";
const GPU_RESET_HELP: &str = "Reset nvml gpu freq to default";
const POWER_HELP: &str = "Set nvml device power limit in watts";
const POWER_RESET_HELP: &str = "Reset nvml power limit to default";

#[rustfmt::skip]
fn card_help_long() -> String {
"Target nvml drm card indexes or bus ids, comma-delimited
Bus id syntax: BUS:ID e.g. pci:0000:00:02.0".to_string()
}

fn gpu_min_help_long() -> String {
    format!("Set nvml min gpu freq in megahertz per --{}", CARD)
}

fn gpu_max_help_long() -> String {
    format!("Set nvml max gpu freq in megahertz per --{}", CARD)
}

fn gpu_reset_help_long() -> String {
    format!("Reset nvml gpu freq to default per --{}", CARD)
}

fn power_help_long() -> String {
    format!("Set nvml device power limit in watts per --{}", CARD)
}

fn power_reset_help_long() -> String {
    format!("Reset nvml power limit to default per --{}", CARD)
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
            name: GPU_MIN.into(),
            long: GPU_MIN.into(),
            short: GPU_MIN_SHORT.into(),
            value_name: "INT".into(),
            help: GPU_MIN_HELP.into(),
            help_long: gpu_min_help_long().into(),
            requires: vec![CARD].into(),
            ..Default::default()
        },
        Arg {
            name: GPU_MAX.into(),
            long: GPU_MAX.into(),
            short: GPU_MAX_SHORT.into(),
            value_name: "INT".into(),
            help: GPU_MAX_HELP.into(),
            help_long: gpu_max_help_long().into(),
            requires: vec![CARD].into(),
            ..Default::default()
        },
        Arg {
            name: GPU_RESET.into(),
            long: GPU_RESET.into(),
            short: GPU_RESET_SHORT.into(),
            help: GPU_RESET_HELP.into(),
            help_long: gpu_reset_help_long().into(),
            requires: vec![CARD].into(),
            conflicts: vec![GPU_MIN, GPU_MAX].into(),
            ..Default::default()
        },
        Arg {
            name: POWER.into(),
            long: POWER.into(),
            short: POWER_SHORT.into(),
            value_name: "FLOAT".into(),
            help: POWER_HELP.into(),
            help_long: power_help_long().into(),
            requires: vec![CARD].into(),
            ..Default::default()
        },
        Arg {
            name: POWER_RESET.into(),
            long: POWER_RESET.into(),
            short: POWER_RESET_SHORT.into(),
            help: POWER_RESET_HELP.into(),
            help_long: power_reset_help_long().into(),
            requires: vec![CARD].into(),
            conflicts: vec![POWER].into(),
            ..Default::default()
        },
    ]
}

impl super::Values {
    pub(super) async fn from_parser(p: Parser<'_>) -> Result<Self> {
        log::trace!("nvml parse start");
        let r = Self {
            cards: p.drm_ids::<NvmlDriver>(CARD).await?,
            gpu_min: p.megahertz(GPU_MIN)?,
            gpu_max: p.megahertz(GPU_MAX)?,
            gpu_reset: p.flag(GPU_RESET),
            power: p.watts(POWER)?,
            power_reset: p.flag(POWER_RESET),
        };
        log::trace!("nvml parse done");
        Ok(r)
    }
}
