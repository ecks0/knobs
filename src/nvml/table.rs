use futures::stream::TryStreamExt as _;
use measurements::{Frequency, Power};
use syx::nvml::Values as Card;

use crate::util::format::{dot, frequency, power, Table};

fn mhz(mhz: u32) -> String {
    frequency(Frequency::from_megahertz(mhz as f64))
}

fn mw(mw: u32) -> String {
    // Do not show decimal places for power usage.
    let p = Power::from_milliwatts(mw as f64);
    let p = p.as_watts().trunc();
    let p = Power::from_watts(p);
    power(p)
}

pub(super) async fn tabulate() -> Option<String> {
    let mut cards: Vec<_> = Card::all().try_collect().await.unwrap_or_default();
    if cards.is_empty() {
        None
    } else {
        cards.sort_by_key(|v| v.id());
        let mut tab = Table::new(&[
            "DRM ",
            "Driver",
            "GPU cur",
            "GPU lim",
            "Power cur",
            "Power lim",
            "Min lim",
            "Max lim",
        ]);
        for card in cards {
            tab.row(&[
                card.id().to_string(),
                "nvidia".to_string(),
                card.gfx_freq().await.ok().map(mhz).unwrap_or_else(dot),
                card.gfx_max_freq().await.ok().map(mhz).unwrap_or_else(dot),
                card.power().await.ok().map(mw).unwrap_or_else(dot),
                card.power_limit().await.ok().map(mw).unwrap_or_else(dot),
                card.power_min_limit()
                    .await
                    .ok()
                    .map(mw)
                    .unwrap_or_else(dot),
                card.power_max_limit()
                    .await
                    .ok()
                    .map(mw)
                    .unwrap_or_else(dot),
            ])
        }
        Some(tab.to_string())
    }
}
