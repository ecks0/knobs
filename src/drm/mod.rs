mod table;

use futures::Future;

#[derive(Debug)]
pub(crate) struct Drm;

impl Drm {
    pub(crate) fn tabulate() -> impl Future<Output = Option<Vec<String>>> {
        table::tabulate()
    }
}
