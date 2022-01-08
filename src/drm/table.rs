use std::collections::HashSet;

use futures::future::try_join_all;
use futures::stream::{self, StreamExt as _, TryStreamExt as _};
use syx::drm::Cache as Card;

use crate::util::format::{dot, Table};
use crate::{Nvml, I915};

async fn id_driver(cards: Vec<Card>) -> Option<Vec<String>> {
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
        let r = Some(vec![tab.into()]);
        log::trace!("drm tabulate id_driver done");
        r
    }
}

pub(super) async fn tabulate() -> Option<Vec<String>> {
    log::trace!("drm tabulate start");
    let mut cards: Vec<_> = Card::all().try_collect().await.unwrap_or_default();
    if cards.is_empty() {
        log::trace!("drm tabulate none");
        None
    } else {
        cards.sort_by_key(|v| v.id());
        let mut tabulators = vec![tokio::spawn(id_driver(cards.clone()))];
        stream::iter(&cards)
            .filter_map(|card| async move { card.driver().await.ok() })
            .fold(
                // build an ordered list of unique drivers
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
            .for_each(|driver| {
                let t = match driver.as_str() {
                    "i915" => tokio::spawn(I915::tabulate(cards.clone())),
                    "nvidia" => tokio::spawn(Nvml::tabulate(cards.clone())),
                    _ => return,
                };
                tabulators.push(t);
            });
        let tables: Vec<_> = try_join_all(tabulators)
            .await
            .expect("drm tabulate futures")
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        let r = if tables.is_empty() { None } else { Some(tables) };
        log::trace!("drm tabulate done");
        r
    }
}
