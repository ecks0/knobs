use futures::stream::TryStreamExt as _;
use syx::drm::Cache as DrmCard;
use tokio::sync::OnceCell;

pub(crate) async fn cpu_ids() -> Vec<u64> {
    static CPU_IDS: OnceCell<Vec<u64>> = OnceCell::const_new();
    CPU_IDS
        .get_or_init(|| async {
            let mut v: Vec<_> = syx::cpu::ids().try_collect().await.unwrap_or_default();
            v.sort_unstable();
            v
        })
        .await
        .clone()
}

pub(crate) async fn drm_cards() -> Vec<DrmCard> {
    static DRM_CARDS: OnceCell<Vec<DrmCard>> = OnceCell::const_new();
    DRM_CARDS
        .get_or_init(|| async {
            let mut v: Vec<_> = DrmCard::all().try_collect().await.unwrap_or_default();
            v.sort_unstable_by_key(|c| c.id());
            v
        })
        .await
        .clone()
}
