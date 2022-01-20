use std::collections::HashSet;

use futures::future::FutureExt as _;
use futures::stream::{self, StreamExt as _};
use syx::drm::Cache as Card;

use crate::applet::i915::I915;
use crate::applet::nvml::Nvml;
use crate::applet::{Applet as _, Formatter};
use crate::util::format::{dot, Table};
use crate::util::once;

async fn table(cards: Vec<Card>) -> Option<String> {
    log::trace!("drm summary table start");
    if cards.is_empty() {
        log::trace!("drm summary table none");
        None
    } else {
        let mut tab = Table::new(&["DRM", "Driver", "Bus", "Bus id"]);
        for card in cards {
            let (bus, bus_id) =
                card.bus_id().await.ok().map(|v| (Some(v.bus), Some(v.id))).unwrap_or((None, None));
            tab.row([
                card.id().to_string(),
                card.driver().await.ok().unwrap_or_else(dot),
                bus.unwrap_or_else(dot),
                bus_id.unwrap_or_else(dot),
            ]);
        }
        let r = Some(tab.into());
        log::trace!("drm summary table done");
        r
    }
}

pub(super) async fn summary() -> Vec<Formatter> {
    log::trace!("drm summary start");
    let cards = once::drm_cards().await;
    if cards.is_empty() {
        log::trace!("drm summary none");
        return vec![];
    } else {
        let order: Vec<_> = stream::iter(&cards)
            .filter_map(|card| async move { card.driver().await.ok() })
            .fold(
                (HashSet::new(), Vec::new()),
                |(mut h, mut v), driver| async move {
                    if h.insert(driver.clone()) {
                        v.push(driver);
                    }
                    (h, v)
                },
            )
            .await
            .1;
        let mut formatters = vec![table(cards).boxed()];
        for driver in order {
            match driver.as_str() {
                "i915" => formatters.extend(I915::default().summary().await),
                "nvidia" => formatters.extend(Nvml::default().summary().await),
                _ => continue,
            }
        }
        log::trace!("drm summary done");
        formatters
    }
}
