use async_trait::async_trait;

use crate::cli::{Arg, NvmlDriver, Parser};
use crate::util::convert::TryFromRef;
use crate::{Error, Result};

const NVML: &str = "nvml";
const NVML_GPU_MIN: &str = "nvml-gpu-min";
const NVML_GPU_MAX: &str = "nvml-gpu-max";
const NVML_GPU_RESET: &str = "nvml-gpu-reset";
const NVML_POWER: &str = "nvml-power";
const NVML_POWER_RESET: &str = "nvml-power-reset";

#[async_trait]
impl TryFromRef<Parser> for super::Nvml {
    type Error = Error;

    async fn try_from_ref(p: &Parser) -> Result<Self> {
        //log::trace!("nvml parse start");
        let r = Self {
            nvml: p.drm_ids::<NvmlDriver>(NVML).await?,
            nvml_gpu_min: p.megahertz(NVML_GPU_MIN)?,
            nvml_gpu_max: p.megahertz(NVML_GPU_MAX)?,
            nvml_gpu_reset: p.flag(NVML_GPU_RESET),
            nvml_power: p.watts(NVML_POWER)?,
            nvml_power_reset: p.flag(NVML_POWER_RESET),
        };
        //log::trace!("nvml parse done");
        Ok(r)
    }
}

pub(super) fn args() -> Vec<Arg> {
    vec![
        Arg {
            name: NVML.into(),
            long: NVML.into(),
            value_name: "IDS".into(),
            help: nvml_help().into(),
            help_long: nvml_help_long().into(),
            ..Default::default()
        },
        Arg {
            name: NVML_GPU_MIN.into(),
            long: NVML_GPU_MIN.into(),
            value_name: "INT".into(),
            help: nvml_gpu_min_help().into(),
            help_long: nvml_gpu_min_help_long().into(),
            requires: vec![NVML].into(),
            ..Default::default()
        },
        Arg {
            name: NVML_GPU_MAX.into(),
            long: NVML_GPU_MAX.into(),
            value_name: "INT".into(),
            help: nvml_gpu_max_help().into(),
            help_long: nvml_gpu_max_help_long().into(),
            requires: vec![NVML].into(),
            ..Default::default()
        },
        Arg {
            name: NVML_GPU_RESET.into(),
            long: NVML_GPU_RESET.into(),
            help: nvml_gpu_reset_help().into(),
            help_long: nvml_gpu_reset_help_long().into(),
            requires: vec![NVML].into(),
            conflicts: vec![NVML_GPU_MIN, NVML_GPU_MAX].into(),
            ..Default::default()
        },
        Arg {
            name: NVML_POWER.into(),
            long: NVML_POWER.into(),
            value_name: "FLOAT".into(),
            help: nvml_power_help().into(),
            help_long: nvml_power_help_long().into(),
            requires: vec![NVML].into(),
            ..Default::default()
        },
        Arg {
            name: NVML_POWER_RESET.into(),
            long: NVML_POWER_RESET.into(),
            help: nvml_power_reset_help().into(),
            help_long: nvml_power_reset_help_long().into(),
            requires: vec![NVML].into(),
            conflicts: vec![NVML_POWER].into(),
            ..Default::default()
        },
    ]
}

fn nvml_help() -> String {
    "Target nvml drm card indexes or bus ids".to_string()
}

#[rustfmt::skip]
fn nvml_help_long() -> String {
"Target nvml drm card indexes or bus ids, comma-delimited
Bus id syntax: BUS:ID e.g. pci:0000:00:02.0

".to_string()
}

fn nvml_gpu_min_help() -> String {
    "Set nvml min gpu freq in megahertz".to_string()
}

fn nvml_gpu_min_help_long() -> String {
    format!("Set nvml min gpu freq in megahertz per --{}", NVML)
}

fn nvml_gpu_max_help() -> String {
    "Set nvml max gpu freq in megahertz".to_string()
}

fn nvml_gpu_max_help_long() -> String {
    format!("Set nvml max gpu freq in megahertz per --{}", NVML)
}

fn nvml_gpu_reset_help() -> String {
    "Reset nvml gpu freq to default".to_string()
}

fn nvml_gpu_reset_help_long() -> String {
    format!("Reset nvml gpu freq to default per --{}", NVML)
}

fn nvml_power_help() -> String {
    "Set nvml device power limit in watts".to_string()
}

fn nvml_power_help_long() -> String {
    format!("Set nvml device power limit in watts per --{}", NVML)
}

fn nvml_power_reset_help() -> String {
    "Reset nvml power limit to default".to_string()
}

fn nvml_power_reset_help_long() -> String {
    format!("Reset nvml power limit to default per --{}", NVML)
}
