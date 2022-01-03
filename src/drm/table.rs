use futures::stream::TryStreamExt as _;
use syx::drm::Values as Card;

use crate::util::format::{dot, Table};

pub(super) async fn tabulate() -> Option<String> {
    let mut cards: Vec<_> = Card::all().try_collect().await.unwrap_or_default();
    if cards.is_empty() {
        None
    } else {
        cards.sort_by_key(|v| v.id());
        let mut tab = Table::new(&["DRM", "Bus", "Bus id"]);
        for card in cards {
            let (bus, bus_id) =
                card.bus_id().await.ok().map(|v| (Some(v.bus), Some(v.id))).unwrap_or((None, None));
            tab.row(&[
                card.id().to_string(),
                bus.unwrap_or_else(dot),
                bus_id.unwrap_or_else(dot),
            ]);
        }
        Some(tab.into())
    }
}
