use async_trait::async_trait;

use crate::cli::Parser;
use crate::util::convert::*;
use crate::{util, Cpu, Error, Nvml, Rapl, Result, I915};

#[derive(Debug)]
pub(super) struct Group {
    cpu: Cpu,
    i915: I915,
    nvml: Nvml,
    rapl: Rapl,
}

#[async_trait]
impl TryFromRef<Parser> for Group {
    type Error = Error;

    async fn try_from_ref(v: &Parser) -> Result<Self> {
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

    pub(super) async fn apply(&self) -> Result<()> {
        // Temporarily online all offline CPUs which have policy values to apply.
        let ids = self.cpu_policy_ids();
        let onlined = util::cpu::set_online(ids).await?;
        let r = async {
            for (i, v) in self.0.iter().enumerate() {
                if v.cpu.has_policy_values() {
                    v.cpu.apply_policy().await.map_err(|e| Error::apply_group(e, i + 1))?;
                    util::cpu::wait_for_policy().await;
                }
            }
            Result::Ok(())
        }
        .await;
        if r.is_err() {
            let _ = util::cpu::set_offline(onlined).await;
            r?
        } else {
            util::cpu::set_offline(onlined).await?;
        }
        for (i, v) in self.0.iter().enumerate() {
            async {
                v.cpu.apply_online().await?;
                v.rapl.apply().await?;
                v.i915.apply().await?;
                v.nvml.apply().await?;
                Result::Ok(())
            }
            .await
            .map_err(|e| Error::apply_group(e, i + 1))?;
        }
        Ok(())
    }
}

impl FromIterator<Group> for Groups {
    fn from_iter<T: IntoIterator<Item = Group>>(iter: T) -> Self {
        let v: Vec<_> = iter.into_iter().collect();
        Self(v)
    }
}
