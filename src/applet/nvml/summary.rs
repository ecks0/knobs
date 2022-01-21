use futures::future::{join_all, FutureExt as _};
use measurements::{Frequency, Power};

use crate::applet::Formatter;
use crate::util::format::{dot, frequency, power, Table};
use crate::util::once;

fn mhz(v: u32) -> String {
    frequency(Frequency::from_megahertz(v as f64))
}

fn mw(v: u32) -> String {
    power(Power::from_milliwatts(v as f64))
}

async fn not_found() -> Option<String> {
    Some("No nvml devices found\n".to_string())
}

async fn table() -> Option<String> {
    log::trace!("nvml summary table start");
    let cards = once::drm_cards().await;
    let cards: Vec<_> = join_all(cards.into_iter().map(|card| async move {
        let is_nvml = card.driver().await.ok().map(|v| v == "nvidia").unwrap_or(false);
        if is_nvml { Some(syx::nvml::Values::new(card.id())) } else { None }
    }))
    .await
    .into_iter()
    .flatten()
    .collect();
    if cards.is_empty() {
        log::trace!("nvml summary table none");
        not_found().await
    } else {
        let rows = join_all(cards.into_iter().map(|card| async move {
            [
                card.id().to_string(),
                "nvidia".to_string(),
                card.gfx_freq().await.ok().map(mhz).unwrap_or_else(dot),
                card.gfx_max_freq().await.ok().map(mhz).unwrap_or_else(dot),
                card.power().await.ok().map(mw).unwrap_or_else(dot),
                card.power_limit().await.ok().map(mw).unwrap_or_else(dot),
                card.power_min_limit().await.ok().map(mw).unwrap_or_else(dot),
                card.power_max_limit().await.ok().map(mw).unwrap_or_else(dot),
            ]
        }))
        .await;
        let mut tab = Table::new(&[
            "DRM",
            "Driver",
            "GPU cur",
            "GPU lim",
            "Power cur",
            "Power lim",
            "Min lim",
            "Max lim",
        ]);
        tab.rows(rows);
        let r = Some(tab.into());
        log::trace!("nvml summary table done");
        r
    }
}

pub(super) async fn summary() -> Vec<Formatter> {
    log::trace!("nvml summary start");
    let formatters = vec![table().boxed()];
    log::trace!("nvml summary done");
    formatters
}
