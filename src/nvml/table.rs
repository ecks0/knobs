use measurements::{Frequency, Power};
use syx::drm::Cache as DrmCard;
use syx::nvml::Values as Card;
use tokio::spawn;
use tokio::task::JoinHandle;

use crate::util::drm::ids_for_driver;
use crate::util::format::{dot, frequency, power, Table};

fn mhz(v: u32) -> String {
    frequency(Frequency::from_megahertz(v as f64))
}

fn mw(v: u32) -> String {
    power(Power::from_milliwatts(v as f64))
}

pub(super) async fn render(drm_cards: Vec<DrmCard>) -> Option<String> {
    log::trace!("nvml tabulate start");
    let cards: Vec<_> =
        ids_for_driver(drm_cards, "nvidia").await.into_iter().map(Card::new).collect();
    if cards.is_empty() {
        log::trace!("nvml tabulate none");
        None
    } else {
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
        for card in cards {
            tab.row(&[
                card.id().to_string(),
                "nvidia".to_string(),
                card.gfx_freq().await.ok().map(mhz).unwrap_or_else(dot),
                card.gfx_max_freq().await.ok().map(mhz).unwrap_or_else(dot),
                card.power().await.ok().map(mw).unwrap_or_else(dot),
                card.power_limit().await.ok().map(mw).unwrap_or_else(dot),
                card.power_min_limit().await.ok().map(mw).unwrap_or_else(dot),
                card.power_max_limit().await.ok().map(mw).unwrap_or_else(dot),
            ]);
        }
        let r = Some(tab.into());
        log::trace!("nvml tabulate done");
        r
    }
}

pub(super) async fn tabulate(drm_cards: Vec<DrmCard>) -> Vec<JoinHandle<Option<String>>> {
    vec![spawn(render(drm_cards))]
}
