use futures::stream::{iter, StreamExt as _, TryStreamExt as _};
use measurements::Frequency;
use syx::cpu::Cache as Cpu;
use syx::cpufreq::Cache as Cpufreq;
use syx::pstate::policy::Cache as PstatePolicy;
use syx::pstate::system::Cache as PstateSystem;

use crate::util::format::{dot, frequency, Table, DOT};

fn khz(v: u64) -> String {
    frequency(Frequency::from_kilohertz(v as f64))
}

async fn cpu_cpufreq(cpus: &[Cpu], cpufreqs: &[Cpufreq]) -> Option<String> {
    if cpus.is_empty() {
        None
    } else {
        let mut tab = Table::new(&[
            "CPU ",
            "Online",
            "Governor",
            "Cur ",
            "Min ",
            "Max ",
            "Min limit",
            "Max limit",
        ]);
        for cpu in cpus {
            let mut row = vec![
                cpu.id().to_string(),
                cpu.online().await.ok().map(|v| v.to_string()).unwrap_or_else(dot),
            ];
            if let Some(cpufreq) = cpufreqs.iter().find(|v| v.id() == cpu.id()) {
                row.extend([
                    cpufreq.scaling_governor().await.ok().unwrap_or_else(dot),
                    cpufreq.scaling_cur_freq().await.ok().map(khz).unwrap_or_else(dot),
                    cpufreq.scaling_min_freq().await.ok().map(khz).unwrap_or_else(dot),
                    cpufreq.scaling_max_freq().await.ok().map(khz).unwrap_or_else(dot),
                    cpufreq.cpuinfo_min_freq().await.ok().map(khz).unwrap_or_else(dot),
                    cpufreq.cpuinfo_max_freq().await.ok().map(khz).unwrap_or_else(dot),
                ]);
            } else {
                row.extend([dot(), dot(), dot(), dot(), dot(), dot()]);
            }
            tab.row(&row);
        }
        Some(tab.to_string())
    }
}

async fn governors(cpufreqs: &[Cpufreq]) -> Option<String> {
    if cpufreqs.is_empty() {
        None
    } else {
        let mut govs: Vec<String> = iter(cpufreqs.iter())
            .then(|v| async move {
                v.scaling_available_governors().await.ok().map(|g| g.join(" ")).unwrap_or_else(dot)
            })
            .collect()
            .await;
        govs.sort_unstable();
        govs.dedup();
        if govs.is_empty() || (govs.len() == 1 && govs[0] == DOT) {
            None
        } else {
            let mut tab = Table::new(&["CPU ", "Available governors"]);
            if govs.len() == 1 {
                tab.row(&["all", govs[0].as_str()]);
            } else {
                for cpufreq in cpufreqs {
                    tab.row(&[
                        cpufreq.id().to_string(),
                        cpufreq
                            .scaling_available_governors()
                            .await
                            .ok()
                            .map(|g| g.join(" "))
                            .unwrap_or_else(dot),
                    ]);
                }
            }
            Some(tab.to_string())
        }
    }
}

async fn pstate_status(system: &PstateSystem) -> Option<String> {
    if !system.is_active().await.unwrap_or(false) {
        // Print the status when not active so that the user
        // knows why they're not seeing the epb/epp tables.
        system.status().await.ok().map(|v| format!(" intel_pstate: {}\n", v))
    } else {
        None
    }
}

async fn epb_epp(system: &PstateSystem, policies: &[PstatePolicy]) -> Option<String> {
    if policies.is_empty() || !system.is_active().await.ok().unwrap_or(false) {
        None
    } else {
        let mut vals: Vec<_> = iter(policies.iter())
            .then(|v| async move {
                let epb = v
                    .energy_perf_bias()
                    .await
                    .ok()
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(dot);
                let epp = v.energy_performance_preference().await.ok().unwrap_or_else(dot);
                (epb, epp)
            })
            .collect()
            .await;
        vals.sort_unstable();
        vals.dedup();
        if vals.is_empty() || (vals.len() == 1 && vals[0] == (dot(), dot())) {
            None
        } else {
            let mut tab = Table::new(&["CPU ", "EP bias", "EP preference"]);
            if vals.len() == 1 {
                let val = vals.remove(0);
                tab.row(&["all", &val.0, &val.1]);
            } else {
                for policy in policies {
                    tab.row(&[
                        policy.id().to_string(),
                        policy
                            .energy_perf_bias()
                            .await
                            .ok()
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(dot),
                        policy.energy_performance_preference().await.ok().unwrap_or_else(dot),
                    ]);
                }
            }
            Some(tab.to_string())
        }
    }
}

async fn epps(system: &PstateSystem, policies: &[PstatePolicy]) -> Option<String> {
    if policies.is_empty() || !system.is_active().await.ok().unwrap_or(false) {
        None
    } else {
        let mut epps: Vec<_> = iter(policies.iter())
            .then(|v| async move {
                v.energy_performance_available_preferences()
                    .await
                    .ok()
                    .map(|p| p.join(" "))
                    .unwrap_or_else(dot)
            })
            .collect()
            .await;
        epps.sort_unstable();
        epps.dedup();
        if epps.is_empty() || (epps.len() == 1 && epps[0] == DOT) {
            None
        } else {
            let mut tab = Table::new(&["CPU ", "Available EP preferences"]);
            if epps.len() == 1 {
                tab.row(&["all", epps[0].as_str()]);
            } else {
                for policy in policies {
                    tab.row(&[
                        policy.id().to_string(),
                        policy
                            .energy_performance_available_preferences()
                            .await
                            .ok()
                            .map(|p| p.join(" "))
                            .unwrap_or_else(dot),
                    ]);
                }
            }
            Some(tab.to_string())
        }
    }
}

pub(super) async fn tabulate() -> Option<String> {
    let mut cpus: Vec<_> = Cpu::all().try_collect().await.unwrap_or_default();
    let mut cpufreqs: Vec<_> = Cpufreq::all().try_collect().await.unwrap_or_default();
    let mut policies: Vec<_> = PstatePolicy::all().try_collect().await.unwrap_or_default();
    let system = PstateSystem::default();
    cpus.sort_by_key(|v| v.id());
    cpufreqs.sort_by_key(|v| v.id());
    policies.sort_by_key(|v| v.id());
    let tables: Vec<_> = [
        cpu_cpufreq(&cpus, &cpufreqs).await,
        governors(&cpufreqs).await,
        pstate_status(&system).await,
        epb_epp(&system, &policies).await,
        epps(&system, &policies).await,
    ]
    .into_iter()
    .flatten()
    .collect();
    if tables.is_empty() { None } else { Some(tables.join("\n")) }
}
