mod args;
mod summary;

use std::time::Duration;

use async_trait::async_trait;
use measurements::Power;

use crate::applet::{Applet, Formatter};
use crate::cli::{Arg, Parser};
use crate::Result;

#[derive(Debug)]
pub(crate) struct ConstraintIds {
    pub(crate) package: u64,
    pub(crate) subzone: Option<u64>,
    pub(crate) constraints: Vec<u64>,
}

#[derive(Debug)]
struct Values {
    constraint_ids: Option<ConstraintIds>,
    limit: Option<Power>,
    window: Option<Duration>,
    quiet: Option<()>,
}

#[derive(Debug, Default)]
pub(crate) struct Rapl {
    quiet: Option<()>,
}

#[async_trait]
impl Applet for Rapl {
    fn name(&self) -> &'static str {
        "rapl"
    }

    fn about(&self) -> &'static str {
        "View or set rapl values"
    }

    fn args(&self) -> Vec<Arg> {
        args::args()
    }

    async fn run(&mut self, p: Parser<'_>) -> Result<()> {
        log::trace!("rapl run start");
        let values = Values::from_parser(p).await?;
        self.quiet = values.quiet;
        if let Some(constraint_ids) = values.constraint_ids {
            for constraint in constraint_ids.constraints {
                let id = (constraint_ids.package, constraint_ids.subzone, constraint);
                if let Some(v) = values.limit {
                    let v = v.as_microwatts().trunc() as u64;
                    syx::intel_rapl::constraint::set_power_limit_uw(id, v).await?;
                }
                if let Some(v) = values.window {
                    let v: u64 = v.as_micros().try_into().unwrap();
                    syx::intel_rapl::constraint::set_time_window_us(id, v).await?;
                }
            }
        }
        log::trace!("rapl run done");
        Ok(())
    }

    async fn summary(&self) -> Vec<Formatter> {
        if self.quiet.is_none() { summary::summary().await } else { vec![] }
    }

    fn default_summary(&self) -> bool {
        true
    }
}
