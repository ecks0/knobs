use futures::future::{join_all, FutureExt as _};
use futures::stream::{self, StreamExt as _};
use measurements::Frequency;
use syx::cpu::Values as Cpu;
use syx::cpufreq::Values as Cpufreq;
use syx::intel_pstate::policy::Values as PstatePolicy;
use syx::intel_pstate::system::Cache as PstateSystem;

use crate::applet::Formatter;
use crate::util::format::{dot, frequency, Table, DOT};
use crate::util::once;

fn khz(v: u64) -> String {
    frequency(Frequency::from_kilohertz(v as f64))
}

async fn cpu_cpufreq(cpus: Vec<Cpu>, mut cpufreqs: Vec<Cpufreq>) -> Option<String> {
    log::trace!("cpu summary cpu_cpufreq start");
    if cpus.is_empty() {
        log::trace!("cpu summary cpu_cpufreq none");
        None
    } else {
        let futs = cpus.into_iter().map(|cpu| {
            let cpufreq = cpufreqs
                .iter()
                .position(|cpufreq| cpufreq.id() == cpu.id())
                .map(|i| cpufreqs.swap_remove(i));
            async move {
                let mut row = vec![
                    cpu.id().to_string(),
                    cpu.online().await.ok().map(|v| v.to_string()).unwrap_or_else(dot),
                ];
                if let Some(cpufreq) = cpufreq {
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
                row
            }
        });
        let mut tab = Table::new(&[
            "CPU", "Online", "Governor", "Cur", "Min", "Max", "Min lim", "Max lim",
        ]);
        for row in join_all(futs).await {
            tab.row(&row);
        }
        let r = Some(tab.into());
        log::trace!("cpu summary cpu_cpufreq done");
        r
    }
}

async fn governors(cpufreqs: Vec<Cpufreq>) -> Option<String> {
    log::trace!("cpu summary governors start");
    if cpufreqs.is_empty() {
        log::trace!("cpu summary governors none");
        None
    } else {
        let values: Vec<_> = stream::iter(cpufreqs.iter())
            .then(|v| async move {
                let id = v.id().to_string();
                let govs = v
                    .scaling_available_governors()
                    .await
                    .ok()
                    .map(|g| g.join(" "))
                    .unwrap_or_else(dot);
                (id, govs)
            })
            .collect()
            .await;
        let mut govs: Vec<_> = values.iter().map(|(_, g)| g.as_str()).collect();
        govs.sort_unstable();
        govs.dedup();
        if govs.is_empty() || (govs.len() == 1 && govs[0] == DOT) {
            log::trace!("cpu governors none 2");
            None
        } else {
            let mut tab = Table::new(&["CPU", "Available governors"]);
            if govs.len() == 1 {
                tab.row(&["all", govs[0]]);
            } else {
                for (id, govs) in values {
                    tab.row(&[id.to_string(), govs]);
                }
            }
            let r = Some(tab.into());
            log::trace!("cpu summary governors done");
            r
        }
    }
}

async fn pstate_status(system: PstateSystem) -> Option<String> {
    log::trace!("cpu summary pstate_status start");
    if system.is_active().await.unwrap_or(false) {
        log::trace!("cpu summary pstate_status none");
        None
    } else {
        // Print the status when not active so that the user
        // knows why they're not seeing the epb/epp tables.
        let r = system.status().await.ok().map(|v| format!(" intel_pstate: {}\n", v));
        log::trace!("cpu summary pstate_status done");
        r
    }
}

async fn epb_epp(system: PstateSystem, pstates: Vec<PstatePolicy>) -> Option<String> {
    log::trace!("cpu summary epb_epp start");
    if pstates.is_empty() || !system.is_active().await.ok().unwrap_or(false) {
        log::trace!("cpu summary epb_epp none");
        None
    } else {
        let values: Vec<_> = stream::iter(pstates.iter())
            .then(|v| async move {
                let id = v.id().to_string();
                let epb = v
                    .energy_perf_bias()
                    .await
                    .ok()
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(dot);
                let epp = v.energy_performance_preference().await.ok().unwrap_or_else(dot);
                (id, epb, epp)
            })
            .collect()
            .await;
        let mut epb_epp: Vec<_> =
            values.iter().map(|(_, epb, epp)| (epb.as_str(), epp.as_str())).collect();
        epb_epp.sort_unstable();
        epb_epp.dedup();
        if epb_epp.is_empty() || (epb_epp.len() == 1 && epb_epp[0] == (DOT, DOT)) {
            log::trace!("cpu summary epb_epp none 2");
            None
        } else {
            let mut tab = Table::new(&["CPU", "EP bias", "EP preference"]);
            if epb_epp.len() == 1 {
                let (epb, epp) = epb_epp[0];
                tab.row(&["all", epb, epp]);
            } else {
                for (id, epb, epp) in values {
                    tab.row(&[id, epb, epp]);
                }
            }
            let r = Some(tab.into());
            log::trace!("cpu summary epb_epp done");
            r
        }
    }
}

async fn epps(system: PstateSystem, pstates: Vec<PstatePolicy>) -> Option<String> {
    log::trace!("cpu summary epps start");
    if pstates.is_empty() || !system.is_active().await.ok().unwrap_or(false) {
        log::trace!("cpu summary epps none");
        None
    } else {
        let values: Vec<_> = stream::iter(pstates.iter())
            .then(|v| async move {
                let id = v.id().to_string();
                let prefs = v
                    .energy_performance_available_preferences()
                    .await
                    .ok()
                    .map(|p| p.join(" "))
                    .unwrap_or_else(dot);
                (id, prefs)
            })
            .collect()
            .await;
        let mut epps: Vec<_> = values.iter().map(|(_, epps)| epps.as_str()).collect();
        epps.sort_unstable();
        epps.dedup();
        if epps.is_empty() || (epps.len() == 1 && epps[0] == DOT) {
            log::trace!("cpu summary epps none 2");
            None
        } else {
            let mut tab = Table::new(&["CPU", "Available EP preferences"]);
            if epps.len() == 1 {
                tab.row(&["all", epps[0]]);
            } else {
                for (id, epps) in values {
                    tab.row(&[id, epps]);
                }
            }
            let r = Some(tab.into());
            log::trace!("cpu summary epps done");
            r
        }
    }
}

pub(super) async fn summary() -> Vec<Formatter> {
    log::trace!("cpu summary start");
    let ids: Vec<_> = once::cpu_ids().await;
    let cpus: Vec<_> = ids.clone().into_iter().map(Cpu::new).collect();
    let cpufreqs: Vec<_> = ids.clone().into_iter().map(Cpufreq::new).collect();
    let pstates: Vec<_> = ids.into_iter().map(PstatePolicy::new).collect();
    let system = PstateSystem::default();
    log::trace!("cpu summary formatters");
    let formatters: Vec<Formatter> = vec![
        cpu_cpufreq(cpus, cpufreqs.clone()).boxed(),
        governors(cpufreqs).boxed(),
        pstate_status(system.clone()).boxed(),
        epb_epp(system.clone(), pstates.clone()).boxed(),
        epps(system, pstates).boxed(),
    ];
    log::trace!("cpu summary done");
    formatters
}
