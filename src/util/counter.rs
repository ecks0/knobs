use std::time::{Duration, Instant};

use once_cell::sync::OnceCell;

pub(crate) fn start() -> Instant {
    static START: OnceCell<Instant> = OnceCell::new();
    *START.get_or_init(Instant::now)
}

pub(crate) fn elapsed() -> Duration {
    let now = Instant::now();
    now - start()
}
