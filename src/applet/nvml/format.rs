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

async fn table() -> Option<String> {
    log::trace!("nvml format table start");
    let cards = once::drm_cards().await;
    let cards: Vec<_> = join_all(cards.into_iter().map(|drm_card| async move {
        let is_nvml = drm_card.driver().await.ok().map(|v| v == "nvidia").unwrap_or(false);
        if is_nvml {
            let id = drm_card.id();
            Some((drm_card, syx::nvml::Values::new(id)))
        } else {
            None
        }
    }))
    .await
    .into_iter()
    .flatten()
    .collect();
    if cards.is_empty() {
        log::trace!("nvml format table none");
        None
    } else {
        let rows = join_all(cards.into_iter().map(|(drm_card, card)| async move {
            [
                drm_card.id().to_string(),
                drm_card.bus_id().await.ok().map(|v| v.id).unwrap_or_else(dot),
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
            "Nvml",
            "Bus id",
            "Gpu cur",
            "Gpu lim",
            "Power cur",
            "Power lim",
            "Min lim",
            "Max lim",
        ]);
        tab.rows(rows);
        let r = Some(tab.into());
        log::trace!("nvml format table done");
        r
    }
}

pub(super) async fn format() -> Vec<Formatter> {
    log::trace!("nvml format start");
    let formatters = vec![table().boxed()];
    log::trace!("nvml format done");
    formatters
}
