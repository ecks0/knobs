use futures::stream::{iter, StreamExt as _, TryStreamExt as _};
use measurements::Frequency;
use syx::cpu::Values as Cpu;
use syx::cpufreq::Cache as Cpufreq;
use syx::pstate::policy::Cache as PstatePolicy;
use syx::pstate::system::Cache as PstateSystem;

use crate::util::format::{dot, frequency, Table, DOT};

fn khz(v: u64) -> String {
    frequency(Frequency::from_kilohertz(v as f64))
}

async fn cpu_cpufreq(cpus: Vec<Cpu>, cpufreqs: Vec<Cpufreq>) -> Option<String> {
    log::trace!("cpu cpu_cpufreq start");
    if cpus.is_empty() {
        log::trace!("cpu cpu_cpufreq none");
        None
    } else {
        let mut tab = Table::new(&[
            "CPU", "Online", "Governor", "Cur", "Min", "Max", "Min lim", "Max lim",
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
        let r = Some(tab.into());
        log::trace!("cpu cpu_cpufreq done");
        r
    }
}

async fn governors(cpufreqs: Vec<Cpufreq>) -> Option<String> {
    log::trace!("cpu governors start");
    if cpufreqs.is_empty() {
        log::trace!("cpu governors none");
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
            log::trace!("cpu governors none 2");
            None
        } else {
            let mut tab = Table::new(&["CPU", "Available governors"]);
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
            let r = Some(tab.into());
            log::trace!("cpu governors done");
            r
        }
    }
}

async fn pstate_status(system: PstateSystem) -> Option<String> {
    log::trace!("cpu pstate_status start");
    if system.is_active().await.unwrap_or(false) {
        log::trace!("cpu pstate_status none");
        None
    } else {
        // Print the status when not active so that the user
        // knows why they're not seeing the epb/epp tables.
        let r = system.status().await.ok().map(|v| format!(" intel_pstate: {}\n", v));
        log::trace!("cpu pstate_status done");
        r
    }
}

async fn epb_epp(system: PstateSystem, policies: Vec<PstatePolicy>) -> Option<String> {
    log::trace!("cpu epb_epp start");
    if policies.is_empty() || !system.is_active().await.ok().unwrap_or(false) {
        log::trace!("cpu epb_epp none");
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
            log::trace!("cpu epb_epp none 2");
            None
        } else {
            let mut tab = Table::new(&["CPU", "EP bias", "EP preference"]);
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
            let r = Some(tab.into());
            log::trace!("cpu epb_epp done");
            r
        }
    }
}

async fn epps(system: PstateSystem, policies: Vec<PstatePolicy>) -> Option<String> {
    log::trace!("cpu epps start");
    if policies.is_empty() || !system.is_active().await.ok().unwrap_or(false) {
        log::trace!("cpu epps none");
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
            log::trace!("cpu epps none 2");
            None
        } else {
            let mut tab = Table::new(&["CPU", "Available EP preferences"]);
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
            let r = Some(tab.into());
            log::trace!("cpu epps done");
            r
        }
    }
}

pub(super) async fn tabulate() -> Option<String> {
    log::trace!("cpu tabulate start");
    let mut cpus: Vec<_> = Cpu::all().try_collect().await.unwrap_or_default();
    let mut cpufreqs: Vec<_> = Cpufreq::all().try_collect().await.unwrap_or_default();
    let mut policies: Vec<_> = PstatePolicy::all().try_collect().await.unwrap_or_default();
    let system = PstateSystem::default();
    log::trace!("cpu tabulate sort");
    cpus.sort_by_key(|v| v.id());
    cpufreqs.sort_by_key(|v| v.id());
    policies.sort_by_key(|v| v.id());
    log::trace!("cpu tabulate spawn");
    let tabulators: Vec<_> = vec![
        tokio::spawn(cpu_cpufreq(cpus, cpufreqs.clone())),
        tokio::spawn(governors(cpufreqs)),
        tokio::spawn(pstate_status(system.clone())),
        tokio::spawn(epb_epp(system.clone(), policies.clone())),
        tokio::spawn(epps(system, policies)),
    ];
    log::trace!("cpu tabulate join");
    let tables: Vec<_> = futures::future::join_all(tabulators)
        .await
        .into_iter()
        .map(|v| v.expect("tabulate cpu future"))
        .flatten()
        .collect();
    let r = if tables.is_empty() { None } else { Some(tables.join("\n")) };
    log::trace!("cpu tabulate done");
    r
}
