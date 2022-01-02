use std::time::Duration;

use futures::future::try_join_all;
use futures::stream::TryStreamExt as _;
use measurements::Power;
use syx::rapl::constraint::{Values as Constraint, LONG_TERM, SHORT_TERM};
use syx::rapl::zone::{self, Id as ZoneId, Values as Zone};
use tokio::time::sleep;

use crate::util::format::{dot, power, Table};

fn uw(v: u64) -> String {
    power(Power::from_microwatts(v as f64))
}

fn us(us: u64) -> String {
    format!("{} μs", us)
}

fn format_zone_id(id: ZoneId) -> String {
    if let Some(subzone) = id.subzone() {
        format!("{}:{}", id.package(), subzone)
    } else {
        format!("{}", id.package())
    }
}

async fn limit_window(zone: &Zone, constraint: &str) -> (Option<u64>, Option<u64>) {
    if let Ok(Some(c)) = Constraint::for_name(zone.id(), constraint).await {
        (c.power_limit_uw().await.ok(), c.time_window_us().await.ok())
    } else {
        (None, None)
    }
}

async fn usage(zone: ZoneId) -> (ZoneId, Option<u64>) {
    const INTERVAL: Duration = Duration::from_millis(200);
    const SCALE: u64 = 1000 / INTERVAL.as_millis() as u64;

    if let Ok(a) = zone::energy_uj(zone).await {
        sleep(INTERVAL).await;
        if let Ok(b) = zone::energy_uj(zone).await {
            let v = b - a;
            let v = v * SCALE;
            return (zone, Some(v));
        }
    }
    (zone, None)
}

async fn usages(zones: &[Zone]) -> Vec<(ZoneId, Option<u64>)> {
    let f = zones.iter().map(|v| usage(v.id()));
    let f: Vec<_> = f.map(tokio::spawn).collect();
    try_join_all(f)
        .await
        .expect("join rapl usage sampler tasks")
}

pub(super) async fn tabulate() -> Option<String> {
    let mut zones: Vec<_> = Zone::all().try_collect().await.unwrap_or_default();
    if zones.is_empty() {
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
        let usages = usages(&zones).await;
        for zone in zones {
            let (long_lim, long_win) = limit_window(&zone, LONG_TERM).await;
            let (short_lim, short_win) = limit_window(&zone, SHORT_TERM).await;
            let usage = usages.iter().find(|v| v.0 == zone.id()).and_then(|v| v.1);
            tab.row(&[
                format_zone_id(zone.id()),
                zone.name().await.ok().map(String::from).unwrap_or_else(dot),
                long_lim.map(uw).unwrap_or_else(dot),
                short_lim.map(uw).unwrap_or_else(dot),
                long_win.map(us).unwrap_or_else(dot),
                short_win.map(us).unwrap_or_else(dot),
                usage.map(uw).unwrap_or_else(dot),
            ]);
        }
        Some(tab.to_string())
    }
}
