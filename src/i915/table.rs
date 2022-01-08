use measurements::Frequency;
use syx::drm::Cache as DrmCard;
use syx::i915::Values as Card;

use crate::util::drm::ids_for_driver;
use crate::util::format::{dot, frequency, Table};

fn mhz(v: u64) -> String {
    frequency(Frequency::from_megahertz(v as f64))
}

pub(super) async fn tabulate(drm_cards: Vec<DrmCard>) -> Option<Vec<String>> {
    log::trace!("i915 tabulate start");
    let cards: Vec<_> =
        ids_for_driver(drm_cards, "i915").await.into_iter().map(Card::new).collect();
    if cards.is_empty() {
        log::trace!("i915 tabulate none");
        None
    } else {
        let mut tab = Table::new(&[
            "DRM", "Driver", "Actual", "Req'd", "Min", "Max", "Boost", "Min lim", "Max lim",
        ]);
        for card in cards {
            tab.row(&[
                card.id().to_string(),
                "i915".to_string(),
                card.act_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.cur_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.min_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.max_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.boost_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.rpn_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
                card.rp0_freq_mhz().await.ok().map(mhz).unwrap_or_else(dot),
            ]);
        }
        let r = Some(vec![tab.into()]);
        log::trace!("i915 tabulate done");
        r
    }
}
