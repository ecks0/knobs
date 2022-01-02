use futures::stream::TryStreamExt as _;
use measurements::Power;
use syx::rapl::constraint::{Values as Constraint, LONG_TERM, SHORT_TERM};
use syx::rapl::zone::{Id as ZoneId, Values as Zone};

use crate::util::format::{dot, power, Table};

fn mw(v: u64) -> String {
    power(Power::from_milliwatts(v as f64))
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

pub(super) async fn tabulate() -> Option<String> {
    let zones: Vec<_> = Zone::all().try_collect().await.unwrap_or_default();
    if zones.is_empty() {
        None
    } else {
        let mut tab = Table::new(&[
            "Zone name",
            "Zone",
            "Long limit",
            "Short limit",
            "Long window",
            "Short window",
        ]);
        for zone in zones {
            let (long_lim, long_win) = limit_window(&zone, LONG_TERM).await;
            let (short_lim, short_win) = limit_window(&zone, SHORT_TERM).await;
            tab.row(&[
                zone.name().await.ok().map(String::from).unwrap_or_else(dot),
                format_zone_id(zone.id()),
                long_lim.map(mw).unwrap_or_else(dot),
                short_lim.map(mw).unwrap_or_else(dot),
                long_win.map(us).unwrap_or_else(dot),
                short_win.map(us).unwrap_or_else(dot),
            ]);
        }
        Some(tab.to_string())
    }
}
