use futures::future::{join_all, FutureExt as _};
use measurements::Frequency;

use crate::applet::Formatter;
use crate::util::format::{dot, frequency, Table};
use crate::util::once;

fn mhz(v: u64) -> String {
    frequency(Frequency::from_megahertz(v as f64))
}

async fn table() -> Option<String> {
    log::trace!("i915 format table start");
    let drm_cards = once::drm_cards().await;
    let cards: Vec<_> = join_all(drm_cards.into_iter().map(|drm_card| async move {
        let is_i915 = drm_card.driver().await.ok().map(|v| v == "i915").unwrap_or(false);
        if is_i915 {
            let id = drm_card.id();
            Some((drm_card, syx::i915::Values::new(id)))
        } else {
            None
        }
    }))
    .await
    .into_iter()
    .flatten()
    .collect();
    if cards.is_empty() {
        log::trace!("i915 format table none");
        None
    } else {
        let rows = join_all(cards.into_iter().map(|(drm_card, card)| async move {
            [
                drm_card.id().to_string(),
                drm_card.bus_id().await.ok().map(|v| v.id).unwrap_or_else(dot),
                card.act_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.min_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.max_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.boost_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.rpn_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.rp0_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
            ]
        }))
        .await;
        let mut tab = Table::new(&[
            "i915",
            "Bus id",
            "Gpu cur",
            "Gpu min",
            "Gpu max",
            "Gpu boost",
            "Min lim",
            "Max lim",
        ]);
        tab.rows(rows);
        let r = Some(tab.into());
        log::trace!("i915 format table done");
        r
    }
}

pub(super) async fn format() -> Vec<Formatter> {
    log::trace!("i915 format start");
    let formatters = vec![table().boxed()];
    log::trace!("i915 format done");
    formatters
}
