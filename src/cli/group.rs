use async_trait::async_trait;

use crate::cli::{Parser, ARGS, NAME};
use crate::util::convert::*;
use crate::{util, Cpu, Error, Nvml, Rapl, Result, I915};

#[derive(Debug)]
struct Group {
    cpu: Cpu,
    i915: I915,
    nvml: Nvml,
    rapl: Rapl,
}

#[async_trait]
impl<'a> TryFromRef<Parser<'a>> for Group {
    type Error = Error;

    async fn try_from_ref(v: &Parser<'a>) -> Result<Self> {
        let r = Self {
            cpu: Cpu::try_from_ref(v).await?,
            i915: I915::try_from_ref(v).await?,
            nvml: Nvml::try_from_ref(v).await?,
            rapl: Rapl::try_from_ref(v).await?,
        };
        Ok(r)
    }
}

#[derive(Debug)]
pub(super) struct Groups(Vec<Group>);

impl Groups {
    pub(super) fn has_cpu_values(&self) -> bool {
        self.0.iter().any(|v| !v.cpu.is_empty())
    }

    pub(super) fn has_i915_values(&self) -> bool {
        self.0.iter().any(|v| !v.i915.is_empty())
    }

    pub(super) fn has_nvml_values(&self) -> bool {
        self.0.iter().any(|v| !v.nvml.is_empty())
    }

    pub(super) fn has_rapl_values(&self) -> bool {
        self.0.iter().any(|v| !v.rapl.is_empty())
    }

    fn cpu_policy_ids(&self) -> Vec<u64> {
        let mut r = self
            .0
            .iter()
            .filter_map(
                |v| {
                    if v.cpu.has_policy_values() { v.cpu.cpu.clone() } else { None }
                },
            )
            .flatten()
            .collect::<Vec<_>>();
        r.sort_unstable();
        r.dedup();
        r
    }

    async fn apply_cpu_policies(&self) -> Result<()> {
        // Typically a CPU must be online in order to modify its policy settings,
        // so we will temporarily online all CPU ids which have policy values.
        let ids = self.cpu_policy_ids();
        let onlined = util::cpu::set_online(ids).await?;
        for v in &self.0 {
            if v.cpu.has_policy_values() {
                v.cpu.apply_policy().await?;
                util::cpu::wait_for_policy().await;
            }
        }
        util::cpu::set_offline(onlined).await?;
        Ok(())
    }

    pub(super) async fn apply(&self) -> Result<()> {
        self.apply_cpu_policies().await?;
        for v in &self.0 {
            v.cpu.apply_online().await?;
            v.rapl.apply().await?;
            v.i915.apply().await?;
            v.nvml.apply().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> TryFromRef<Parser<'a>> for Groups {
    type Error = Error;

    async fn try_from_ref(p: &Parser<'a>) -> Result<Self> {
        let mut groups = vec![];
        let mut args = p.strings(ARGS);
        groups.push(p.try_ref_into().await?);
        while let Some(mut a) = args {
            a.insert(0, NAME.to_string());
            let p = Parser::new(p.args, &a)?;
            args = p.strings(ARGS);
            groups.push(p.try_ref_into().await?);
        }
        let r = Groups(groups);
        Ok(r)
    }
}
