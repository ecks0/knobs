use futures::stream::{self, StreamExt as _};
use syx::drm::Cache as Card;

pub(crate) async fn ids_for_driver(cards: Vec<Card>, driver: &str) -> Vec<u64> {
    stream::iter(cards)
        .filter_map(|card| async move {
            card.driver().await.ok().and_then(|d| if d == driver { Some(card.id()) } else { None })
        })
        .collect()
        .await
}
