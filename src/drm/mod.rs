mod table;

use tokio::task::JoinHandle;

#[derive(Debug)]
pub(crate) struct Drm;

impl Drm {
    pub(crate) async fn tabulate() -> Vec<JoinHandle<Option<String>>> {
        table::tabulate().await
    }
}
