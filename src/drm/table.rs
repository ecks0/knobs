use std::collections::HashSet;

use futures::stream::{self, StreamExt as _, TryStreamExt as _};
use syx::drm::Values as Card;
use tokio::spawn;
use tokio::task::JoinHandle;

use crate::util::format::{dot, Table};
use crate::{Nvml, I915};

async fn id_driver(cards: Vec<Card>) -> Option<String> {
    log::trace!("drm tabulate id_driver start");
    if cards.is_empty() {
        log::trace!("drm tabulate id_driver none");
        None
    } else {
        let mut tab = Table::new(&["DRM", "Driver", "Bus", "Bus id"]);
        for card in cards {
            let (bus, bus_id) =
                card.bus_id().await.ok().map(|v| (Some(v.bus), Some(v.id))).unwrap_or((None, None));
            tab.row(&[
                card.id().to_string(),
                card.driver().await.ok().unwrap_or_else(dot),
                bus.unwrap_or_else(dot),
                bus_id.unwrap_or_else(dot),
            ]);
        }
        let r = Some(tab.into());
        log::trace!("drm tabulate id_driver done");
        r
    }
}

pub(super) async fn tabulate() -> Vec<JoinHandle<Option<String>>> {
    log::trace!("drm tabulate start");
    let mut cards: Vec<_> = Card::all().try_collect().await.unwrap_or_default();
    cards.sort_by_key(|v| v.id());
    let mut tabulators = vec![spawn(id_driver(cards.clone()))];
    let drivers = stream::iter(&cards)
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
        .1
        .into_iter()
        .map(|v| (&cards, v));
    let drivers: Vec<_> = stream::iter(drivers)
        .filter_map(|(cards, driver)| async move {
            let v = match driver.as_str() {
                "i915" => I915::tabulate(cards.clone()).await,
                "nvidia" => Nvml::tabulate(cards.clone()).await,
                _ => return None,
            };
            Some(v)
        })
        .map(stream::iter)
        .flatten()
        .collect()
        .await;
    tabulators.extend(drivers);
    log::trace!("drm tabulate done");
    tabulators
}
