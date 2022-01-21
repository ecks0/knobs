use futures::future::{join_all, FutureExt as _};
use measurements::Frequency;

use crate::applet::Formatter;
use crate::util::format::{dot, frequency, Table};
use crate::util::once;

fn mhz(v: u64) -> String {
    frequency(Frequency::from_megahertz(v as f64))
}

async fn table() -> Option<String> {
    log::trace!("i915 summary table start");
    let cards = once::drm_cards().await;
    let cards: Vec<_> = join_all(cards.into_iter().map(|card| async move {
        let is_i915 = card.driver().await.ok().map(|v| v == "i915").unwrap_or(false);
        if is_i915 { Some(syx::i915::Values::new(card.id())) } else { None }
    }))
    .await
    .into_iter()
    .flatten()
    .collect();
    if cards.is_empty() {
        log::trace!("i915 summary table none");
        None
    } else {
        let rows = join_all(cards.into_iter().map(|card| async move {
            [
                card.id().to_string(),
                "i915".to_string(),
                card.act_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.cur_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.min_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.max_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.boost_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.rpn_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.rp0_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
            ]
        }))
        .await;
        let mut tab = Table::new(&[
            "DRM", "Driver", "Actual", "Req'd", "Min", "Max", "Boost", "Min lim", "Max lim",
        ]);
        tab.rows(rows);
        let r = Some(tab.into());
        log::trace!("i915 summary table done");
        r
    }
}

pub(super) async fn summary() -> Vec<Formatter> {
    log::trace!("i915 summary start");
    let formatters = vec![table().boxed()];
    log::trace!("i915 summary done");
    formatters
}
