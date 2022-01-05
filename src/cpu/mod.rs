mod args;
mod table;

use futures::Future;
use measurements::Frequency;

use crate::cli::Arg;
use crate::Result;

#[derive(Debug)]
pub(crate) struct Cpu {
    pub(crate) cpu: Option<Vec<u64>>,
    pub(crate) cpu_on: Option<bool>,
    pub(crate) cpu_gov: Option<String>,
    pub(crate) cpu_min: Option<Frequency>,
    pub(crate) cpu_max: Option<Frequency>,
    pub(crate) cpu_epb: Option<u64>,
    pub(crate) cpu_epp: Option<String>,
}

impl Cpu {
    pub(crate) fn args() -> impl IntoIterator<Item = Arg> {
        args::args()
    }

    pub(crate) fn is_empty(&self) -> bool {
        !(self.has_online_values() || self.has_policy_values())
    }

    pub(crate) fn has_online_values(&self) -> bool {
        self.cpu.as_ref().map(|v| !v.is_empty()).unwrap_or(false) && self.cpu_on.is_some()
    }

    pub(crate) fn has_policy_values(&self) -> bool {
        self.cpu.as_ref().map(|v| !v.is_empty()).unwrap_or(false)
            && (self.cpu_gov.is_some()
                || self.cpu_min.is_some()
                || self.cpu_max.is_some()
                || self.cpu_epb.is_some()
                || self.cpu_epp.is_some())
    }

    pub(crate) async fn apply_online(&self) -> Result<()> {
        log::trace!("cpu apply_online start");
        if let Some(cpu) = &self.cpu {
            if let Some(cpu_on) = self.cpu_on {
                for id in cpu {
                    syx::cpu::set_online(*id, cpu_on).await?;
                }
            }
        }
        log::trace!("cpu apply_online done");
        Ok(())
    }

    pub(crate) async fn apply_policy(&self) -> Result<()> {
        log::trace!("cpu apply_policy start");
        if let Some(cpu) = self.cpu.clone() {
            for id in cpu {
                if let Some(v) = &self.cpu_gov {
                    syx::cpufreq::set_scaling_governor(id, v).await?;
                }
                if let Some(v) = self.cpu_min {
                    let v = v.as_kilohertz().trunc() as u64;
                    syx::cpufreq::set_scaling_min_freq(id, v).await?;
                }
                if let Some(v) = self.cpu_max {
                    let v = v.as_kilohertz().trunc() as u64;
                    syx::cpufreq::set_scaling_max_freq(id, v).await?;
                }
                if let Some(v) = self.cpu_epb {
                    syx::pstate::policy::set_energy_perf_bias(id, v).await?;
                }
                if let Some(v) = &self.cpu_epp {
                    syx::pstate::policy::set_energy_performance_preference(id, v).await?;
                }
            }
        }
        log::trace!("cpu apply_policy done");
        Ok(())
    }

    pub(crate) fn tabulate() -> impl Future<Output = Option<String>> {
        table::tabulate()
    }
}
