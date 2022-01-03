use futures::stream::TryStreamExt as _;
use measurements::Frequency;
use syx::i915::Values as Card;

use crate::util::format::{dot, frequency, Table};

fn mhz(v: u64) -> String {
    frequency(Frequency::from_megahertz(v as f64))
}

pub(super) async fn tabulate() -> Option<String> {
    let mut cards: Vec<_> = Card::all().try_collect().await.unwrap_or_default();
    if cards.is_empty() {
        None
    } else {
        cards.sort_by_key(|v| v.id());
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
        Some(tab.into())
    }
}
