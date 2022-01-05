mod args;
mod table;

use std::time::Duration;

use futures::Future;
use measurements::Power;

use crate::cli::Arg;
use crate::Result;

#[derive(Debug)]
pub(crate) struct Rapl {
    pub(crate) rapl_constraint: Option<(u64, Option<u64>, u64)>,
    pub(crate) rapl_limit: Option<Power>,
    pub(crate) rapl_window: Option<Duration>,
}

impl Rapl {
    pub(crate) fn args() -> impl IntoIterator<Item = Arg> {
        args::args()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rapl_constraint.is_none() || (self.rapl_limit.is_none() && self.rapl_window.is_none())
    }

    pub(crate) async fn apply(&self) -> Result<()> {
        log::trace!("rapl apply start");
        if let Some(id) = self.rapl_constraint {
            if let Some(v) = self.rapl_limit {
                let v = v.as_microwatts().trunc() as u64;
                syx::rapl::constraint::set_power_limit_uw(id, v).await?;
            }
            if let Some(v) = self.rapl_window {
                let v: u64 = v.as_micros().try_into().unwrap();
                syx::rapl::constraint::set_time_window_us(id, v).await?;
            }
        }
        log::trace!("rapl apply done");
        Ok(())
    }

    pub(crate) fn tabulate() -> impl Future<Output = Option<String>> {
        table::tabulate()
    }
}
