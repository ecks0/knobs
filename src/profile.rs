use crate::{util, Cpu, Nvml, Rapl, Result, I915};

#[derive(Debug)]
pub(crate) struct Profile {
    pub(crate) cpu: Cpu,
    pub(crate) i915: I915,
    pub(crate) nvml: Nvml,
    pub(crate) rapl: Rapl,
}

#[derive(Debug)]
pub(crate) struct Profiles(pub(crate) Vec<Profile>);

impl Profiles {
    pub(crate) fn has_cpu_values(&self) -> bool {
        self.0.iter().any(|v| !v.cpu.is_empty())
    }

    pub(crate) fn has_i915_values(&self) -> bool {
        self.0.iter().any(|v| !v.i915.is_empty())
    }

    pub(crate) fn has_nvml_values(&self) -> bool {
        self.0.iter().any(|v| !v.nvml.is_empty())
    }

    pub(crate) fn has_rapl_values(&self) -> bool {
        self.0.iter().any(|v| !v.rapl.is_empty())
    }

    fn cpu_policy_ids(&self) -> Vec<u64> {
        let mut r = self
            .0
            .iter()
            .filter_map(|v| {
                if v.cpu.has_policy_values() {
                    v.cpu.cpu.as_ref()
                } else {
                    None
                }
            })
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        r.sort_unstable();
        r.dedup();
        r
    }

    async fn apply_cpu_policies(&self) -> Result<()> {
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

    pub(crate) async fn apply(&self) -> Result<()> {
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
