use std::time::Duration;

use futures::stream::TryStreamExt as _;
use tokio::time::sleep;

use crate::Result;

async fn wait_for_onoff() {
    let millis = 300;
    sleep(Duration::from_millis(millis)).await;
}

async fn wait_for_policy() {
    let millis = 100;
    sleep(Duration::from_millis(millis)).await;
}

async fn set_online(ids: Vec<u64>) -> Result<Vec<u64>> {
    let mut onlined = vec![];
    if !ids.is_empty() {
        let offline: Vec<_> = syx::cpu::offline_ids().try_collect().await?;
        for id in ids {
            if offline.contains(&id) {
                syx::cpu::set_online(id, true).await?;
                onlined.push(id);
            }
        }
    }
    Ok(onlined)
}

async fn set_offline(ids: Vec<u64>) -> Result<Vec<u64>> {
    let mut offlined = vec![];
    if !ids.is_empty() {
        let online: Vec<_> = syx::cpu::online_ids().try_collect().await?;
        for id in ids {
            if online.contains(&id) {
                syx::cpu::set_online(id, false).await?;
                offlined.push(id);
            }
        }
    }
    Ok(offlined)
}

impl super::Values {
    fn has_policy_values(&self) -> bool {
        self.gov.is_some()
            || self.min.is_some()
            || self.max.is_some()
            || self.epb.is_some()
            || self.epp.is_some()
    }
}

pub(super) async fn run(values: super::Values) -> Result<()> {
    log::trace!("cpu run start");
    let has_policy_values = values.has_policy_values();
    if let Some(ids) = values.ids {
        if !ids.is_empty() {
            if has_policy_values {
                let onlined = set_online(ids.clone()).await?;
                if !onlined.is_empty() {
                    wait_for_onoff().await;
                }
                let min = values.min.map(|v| v.as_kilohertz().trunc() as u64);
                let max = values.max.map(|v| v.as_kilohertz().trunc() as u64);
                for id in ids.clone() {
                    if let Some(v) = values.gov.as_ref() {
                        syx::cpufreq::set_scaling_governor(id, v).await?;
                    }
                    if let Some(v) = min {
                        syx::cpufreq::set_scaling_min_freq(id, v).await?;
                    }
                    if let Some(v) = max {
                        syx::cpufreq::set_scaling_max_freq(id, v).await?;
                    }
                    if let Some(v) = values.epb {
                        syx::intel_pstate::policy::set_energy_perf_bias(id, v).await?;
                    }
                    if let Some(v) = values.epp.as_ref() {
                        syx::intel_pstate::policy::set_energy_performance_preference(id, v).await?;
                    }
                }
                wait_for_policy().await;
                if !onlined.is_empty() {
                    set_offline(onlined).await?;
                    wait_for_onoff().await;
                }
            }
            if let Some(on) = values.on {
                for id in ids {
                    syx::cpu::set_online(id, on).await?;
                }
                wait_for_onoff().await;
            }
        }
    }
    log::trace!("cpu run done");
    Ok(())
}
