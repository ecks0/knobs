use std::time::Duration;

use futures::future::try_join_all;
use futures::stream::TryStreamExt as _;
use measurements::Power;
use syx::intel_rapl::constraint::{Values as Constraint, LONG_TERM, SHORT_TERM};
use syx::intel_rapl::zone::{self, Id as ZoneId, Values as Zone};
use tokio::time::sleep;

use crate::util::format::{dot, power, Table};

fn uw(v: u64) -> String {
    power(Power::from_microwatts(v as f64))
}

fn us(v: u64) -> String {
    format!("{} μs", v)
}

fn format_zone_id(v: ZoneId) -> String {
    if let Some(subzone) = v.subzone() {
        format!("{}:{}", v.package(), subzone)
    } else {
        format!("{}", v.package())
    }
}

async fn limit_window(zone: &Zone, constraint: &str) -> (Option<u64>, Option<u64>) {
    if let Ok(Some(c)) = Constraint::for_name(zone.id(), constraint).await {
        (c.power_limit_uw().await.ok(), c.time_window_us().await.ok())
    } else {
        (None, None)
    }
}

async fn energy_uj(zone: ZoneId) -> (ZoneId, Option<u64>) {
    const INTERVAL: Duration = Duration::from_millis(200);
    const SCALE: u64 = 1000 / INTERVAL.as_millis() as u64;

    log::trace!("rapl energy_uj start");
    if let Ok(a) = zone::energy_uj(zone).await {
        sleep(INTERVAL).await;
        if let Ok(b) = zone::energy_uj(zone).await {
            let v = b - a;
            let v = v * SCALE;
            log::trace!("rapl energy_uj done");
            return (zone, Some(v));
        }
    }
    log::trace!("rapl energy_uj done 2");
    (zone, None)
}

async fn energy_ujs(zones: &[Zone]) -> Vec<(ZoneId, Option<u64>)> {
    log::trace!("rapl energy_ujs start");
    let f = zones.iter().map(|v| energy_uj(v.id()));
    let f: Vec<_> = f.map(tokio::spawn).collect();
    let r = try_join_all(f).await.expect("join rapl energy_uj sampler tasks");
    log::trace!("rapl energy_ujs done");
    r
}

pub(super) async fn tabulate() -> Option<Vec<String>> {
    log::trace!("rapl tabulate start");
    let mut zones: Vec<_> = Zone::all().try_collect().await.unwrap_or_default();
    if zones.is_empty() {
        log::trace!("rapl tabulate none");
        None
    } else {
        zones.sort_by_key(|v| v.id());
        let mut tab = Table::new(&[
            "RAPL",
            "Zone name",
            "Long lim",
            "Short lim",
            "Long win",
            "Short win",
            "Usage",
        ]);
        let energy_ujs = energy_ujs(&zones).await;
        for zone in zones {
            let (long_lim, long_win) = limit_window(&zone, LONG_TERM).await;
            let (short_lim, short_win) = limit_window(&zone, SHORT_TERM).await;
            let energy_uj = energy_ujs.iter().find(|v| v.0 == zone.id()).and_then(|v| v.1);
            tab.row(&[
                format_zone_id(zone.id()),
                zone.name().await.ok().unwrap_or_else(dot),
                long_lim.map(uw).unwrap_or_else(dot),
                short_lim.map(uw).unwrap_or_else(dot),
                long_win.map(us).unwrap_or_else(dot),
                short_win.map(us).unwrap_or_else(dot),
                energy_uj.map(uw).unwrap_or_else(dot),
            ]);
        }
        let r = Some(vec![tab.into()]);
        log::trace!("rapl tabulate start");
        r
    }
}
