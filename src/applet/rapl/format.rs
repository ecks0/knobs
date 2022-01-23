use std::time::Duration;

use futures::future::{join_all, FutureExt as _};
use futures::stream::TryStreamExt as _;
use measurements::Power;
use syx::intel_rapl::constraint::{Values as Constraint, LONG_TERM, SHORT_TERM};
use syx::intel_rapl::zone::{self, Id as ZoneId, Values as Zone};
use tokio::time::sleep;

use crate::applet::Formatter;
use crate::util::env;
use crate::util::format::{dot, power, Table};

fn uw(v: u64) -> String {
    power(Power::from_microwatts(v as f64))
}

fn us(v: u64) -> String {
    format!("{} μs", v)
}

fn zone_id(v: ZoneId) -> String {
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

async fn energy_uj(zone: ZoneId, interval: Duration, scale: f64) -> (ZoneId, Option<u64>) {
    log::trace!("rapl format energy_uj start {:?}", zone);
    if let Ok(a) = zone::energy_uj(zone).await {
        sleep(interval).await;
        if let Ok(b) = zone::energy_uj(zone).await {
            let v = ((b - a) as f64 * scale).trunc() as u64;
            log::trace!("rapl format energy_uj done {:?}", zone);
            return (zone, Some(v));
        }
    }
    log::trace!("rapl format energy_uj none {:?}", zone);
    (zone, None)
}

async fn energy_ujs(zones: &[Zone]) -> Vec<(ZoneId, Option<u64>)> {
    const INTERVAL_MS: u64 = 200;

    log::trace!("rapl format energy_ujs start");
    let interval = env::parse::<u64>("RAPL_INTERVAL_MS").unwrap_or(INTERVAL_MS).max(1).min(1000);
    let scale = 1000. / interval as f64;
    let interval = Duration::from_millis(interval);
    let r = join_all(zones.iter().map(|v| energy_uj(v.id(), interval, scale))).await;
    log::trace!("rapl format energy_ujs done");
    r
}

async fn table() -> Option<String> {
    log::trace!("rapl format table start");
    let mut zones: Vec<_> = Zone::all().try_collect().await.unwrap_or_default();
    if zones.is_empty() {
        log::trace!("rapl format table none");
        None
    } else {
        zones.sort_by_key(|v| v.id());
        let energy_ujs = energy_ujs(&zones).await;
        let rows = join_all(zones.into_iter().map(|zone| {
            let energy_uj = energy_ujs.iter().find(|v| v.0 == zone.id()).and_then(|v| v.1);
            async move {
                let (long_lim, long_win) = limit_window(&zone, LONG_TERM).await;
                let (short_lim, short_win) = limit_window(&zone, SHORT_TERM).await;
                [
                    zone_id(zone.id()),
                    zone.name().await.ok().unwrap_or_else(dot),
                    long_lim.map(uw).unwrap_or_else(dot),
                    short_lim.map(uw).unwrap_or_else(dot),
                    long_win.map(us).unwrap_or_else(dot),
                    short_win.map(us).unwrap_or_else(dot),
                    energy_uj.map(uw).unwrap_or_else(dot),
                ]
            }
        }))
        .await;
        let mut tab = Table::new(&[
            "RAPL",
            "Zone name",
            "Long lim",
            "Short lim",
            "Long win",
            "Short win",
            "Usage",
        ]);
        tab.rows(rows);
        let r = Some(tab.into());
        log::trace!("rapl format table done");
        r
    }
}

pub(super) async fn format() -> Vec<Formatter> {
    log::trace!("rapl format start");
    let formatters = vec![table().boxed()];
    log::trace!("rapl format done");
    formatters
}
