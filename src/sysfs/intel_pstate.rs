pub mod path {
    use std::path::PathBuf;

    use crate::sysfs::cpu::path::device_attr as cpu_device_attr;
    use crate::sysfs::cpufreq::path::policy_attr as cpufreq_policy_attr;

    pub fn energy_perf_bias(id: u64) -> PathBuf {
        let mut p = cpu_device_attr(id, "power");
        p.push("energy_perf_bias");
        p
    }

    pub fn energy_performance_preference(id: u64) -> PathBuf {
        cpufreq_policy_attr(id, "energy_performance_preference")
    }

    pub fn energy_performance_available_preferences(id: u64) -> PathBuf {
        cpufreq_policy_attr(id, "energy_performance_available_preferences")
    }

    pub fn device() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu/intel_pstate")
    }

    pub fn device_attr(a: &str) -> PathBuf {
        let mut p = device();
        p.push(a);
        p
    }

    pub fn max_perf_pct() -> PathBuf {
        device_attr("max_perf_pct")
    }

    pub fn min_perf_pct() -> PathBuf {
        device_attr("min_perf_pct")
    }

    pub fn no_turbo() -> PathBuf {
        device_attr("no_turbo")
    }

    pub fn status() -> PathBuf {
        device_attr("status")
    }

    pub fn turbo_pct() -> PathBuf {
        device_attr("turbo_pct")
    }
}

use async_trait::async_trait;

pub use crate::sysfs::cpufreq::policies;
use crate::sysfs::{self, Result};
use crate::{Feature, Resource};

pub async fn energy_perf_bias(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::energy_perf_bias(id)).await
}

pub async fn energy_performance_preference(id: u64) -> Result<String> {
    sysfs::read_str(&path::energy_performance_preference(id)).await
}

pub async fn energy_performance_available_preferences(id: u64) -> Result<Vec<String>> {
    sysfs::read_str_list(&path::energy_performance_available_preferences(id), ' ').await
}

pub async fn max_perf_pct() -> Result<u64> {
    sysfs::read_u64(&path::max_perf_pct()).await
}

pub async fn min_perf_pct() -> Result<u64> {
    sysfs::read_u64(&path::min_perf_pct()).await
}

pub async fn no_turbo() -> Result<bool> {
    sysfs::read_bool(&path::no_turbo()).await
}

pub async fn status() -> Result<String> {
    sysfs::read_str(&path::status()).await
}

pub async fn turbo_pct() -> Result<u64> {
    sysfs::read_u64(&path::turbo_pct()).await
}

pub async fn set_energy_perf_bias(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::energy_perf_bias(id), v).await
}

pub async fn set_energy_performance_preference(id: u64, v: &str) -> Result<()> {
    sysfs::write_str(&path::energy_performance_preference(id), v).await
}

pub async fn set_max_perf_pct(v: u64) -> Result<()> {
    sysfs::write_u64(&path::max_perf_pct(), v).await
}

pub async fn set_min_perf_pct(v: u64) -> Result<()> {
    sysfs::write_u64(&path::min_perf_pct(), v).await
}

pub async fn set_no_turbo(v: bool) -> Result<()> {
    sysfs::write_bool(&path::no_turbo(), v).await
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    pub max_perf_pct: Option<u64>,
    pub min_perf_pct: Option<u64>,
    pub no_turbo: Option<bool>,
    pub status: Option<String>,
    pub turbo_pct: Option<u64>,
}

#[async_trait]
impl Resource for Device {
    type Id = ();
    type Output = Self;

    async fn ids() -> Vec<()> {
        vec![()]
    }

    async fn read(_: ()) -> Option<Self> {
        let max_perf_pct = max_perf_pct().await.ok();
        let min_perf_pct = min_perf_pct().await.ok();
        let no_turbo = no_turbo().await.ok();
        let status = status().await.ok();
        let turbo_pct = turbo_pct().await.ok();
        let s = Self {
            max_perf_pct,
            min_perf_pct,
            no_turbo,
            status,
            turbo_pct,
        };
        if s == Self::default() { None } else { Some(s) }
    }

    async fn write(&self) {
        if let Some(val) = self.max_perf_pct {
            let _ = set_max_perf_pct(val);
        }
        if let Some(val) = self.min_perf_pct {
            let _ = set_min_perf_pct(val);
        }
        if let Some(val) = self.no_turbo {
            let _ = set_no_turbo(val);
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Policy {
    pub id: u64,
    pub energy_perf_bias: Option<u64>,
    pub energy_performance_preference: Option<String>,
    pub energy_performance_available_preferences: Option<Vec<String>>,
}

#[async_trait]
impl Resource for Policy {
    type Id = u64;
    type Output = Self;

    async fn ids() -> Vec<u64> {
        policies().await.ok().unwrap_or_default()
    }

    async fn read(id: u64) -> Option<Self> {
        let energy_perf_bias = energy_perf_bias(id).await.ok();
        let energy_performance_preference = energy_performance_preference(id).await.ok();
        let energy_performance_available_preferences =
            energy_performance_available_preferences(id).await.ok();
        let s = Self {
            id,
            energy_perf_bias,
            energy_performance_preference,
            energy_performance_available_preferences,
        };
        let default = Self {
            id,
            ..Default::default()
        };
        if s == default { None } else { Some(s) }
    }

    async fn write(&self) {
        if let Some(val) = self.energy_perf_bias {
            let _ = set_energy_perf_bias(self.id, val);
        }
        if let Some(val) = &self.energy_performance_preference {
            let _ = set_energy_performance_preference(self.id, val);
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IntelPstate {
    pub device: Option<Device>,
    pub policies: Vec<Policy>,
}

#[async_trait]
impl Feature for IntelPstate {
    async fn present() -> bool {
        path::status().is_file()
    }
}

#[async_trait]
impl Resource for IntelPstate {
    type Id = ();
    type Output = Self;

    async fn ids() -> Vec<()> {
        vec![()]
    }

    async fn read(_: ()) -> Option<Self> {
        let device = Device::read(()).await;
        let policies = Policy::all().await;
        let s = Self { device, policies };
        Some(s)
    }

    async fn write(&self) {
        if let Some(device) = &self.device {
            device.write().await;
        }
        for policy in &self.policies {
            policy.write().await;
        }
    }
}