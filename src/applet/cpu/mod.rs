mod args;
mod summary;

use std::time::Duration;

use async_trait::async_trait;
use futures::stream::TryStreamExt as _;
use measurements::Frequency;
use tokio::time::sleep;

use crate::applet::{Applet, Formatter};
use crate::cli::{Arg, Parser};
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

#[derive(Debug)]
struct Values {
    ids: Option<Vec<u64>>,
    on: Option<bool>,
    gov: Option<String>,
    min: Option<Frequency>,
    max: Option<Frequency>,
    epb: Option<u64>,
    epp: Option<String>,
    quiet: Option<()>,
}

impl Values {
    fn has_policy_values(&self) -> bool {
        self.gov.is_some()
            || self.min.is_some()
            || self.max.is_some()
            || self.epb.is_some()
            || self.epp.is_some()
    }
}

#[derive(Debug, Default)]
pub(crate) struct Cpu {
    quiet: Option<()>,
}

#[async_trait]
impl Applet for Cpu {
    fn name(&self) -> &'static str {
        "cpu"
    }

    fn bin(&self) -> Option<&'static str> {
        Some("kcpu")
    }

    fn about(&self) -> &'static str {
        "View or set cpu values"
    }

    fn args(&self) -> Vec<Arg> {
        args::args()
    }

    async fn run(&mut self, p: Parser<'_>) -> Result<()> {
        log::trace!("cpu run start");
        let values = Values::from_parser(p).await?;
        let has_policy_values = values.has_policy_values();
        self.quiet = values.quiet;
        if let Some(ids) = values.ids {
            if has_policy_values {
                let onlined = set_online(ids.clone()).await?;
                if !onlined.is_empty() {
                    wait_for_onoff().await;
                }
                for id in ids.clone() {
                    if let Some(v) = values.gov.as_ref() {
                        syx::cpufreq::set_scaling_governor(id, v).await?;
                    }
                    if let Some(v) = values.min {
                        let v = v.as_kilohertz().trunc() as u64;
                        syx::cpufreq::set_scaling_min_freq(id, v).await?;
                    }
                    if let Some(v) = values.max {
                        let v = v.as_kilohertz().trunc() as u64;
                        syx::cpufreq::set_scaling_max_freq(id, v).await?;
                    }
                    if let Some(v) = values.epb {
                        syx::intel_pstate::policy::set_energy_perf_bias(id, v).await?;
                    }
                    if let Some(v) = values.epp.as_ref() {
                        syx::intel_pstate::policy::set_energy_performance_preference(id, v)
                            .await?;
                    }
                }
                if !onlined.is_empty() || values.on.is_some() {
                    wait_for_policy().await;
                }
                if !onlined.is_empty() {
                    set_offline(onlined).await?;
                    if values.on.is_some() {
                        wait_for_onoff().await;
                    }
                }
            }
            if let Some(on) = values.on {
                for id in ids {
                    syx::cpu::set_online(id, on).await?;
                }
            }
        }
        log::trace!("cpu run done");
        Ok(())
    }

    async fn summary(&self) -> Vec<Formatter> {
        if self.quiet.is_none() { summary::summary().await } else { vec![] }
    }

    fn default_summary(&self) -> bool {
        true
    }
}
