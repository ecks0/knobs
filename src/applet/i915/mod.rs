mod args;
mod summary;

use async_trait::async_trait;
use measurements::Frequency;

use crate::applet::{Applet, Formatter};
use crate::cli::{Arg, Parser};
use crate::Result;

#[derive(Debug)]
struct Values {
    ids: Option<Vec<u64>>,
    min: Option<Frequency>,
    max: Option<Frequency>,
    boost: Option<Frequency>,
    quiet: Option<()>,
}

#[derive(Debug, Default)]
pub(crate) struct I915 {
    quiet: Option<()>,
}

#[async_trait]
impl Applet for I915 {
    fn name(&self) -> &'static str {
        "915"
    }

    fn bin(&self) -> Option<&'static str> {
        Some("k915")
    }

    fn about(&self) -> &'static str {
        "View or set i915 values"
    }

    fn args(&self) -> Vec<Arg> {
        args::args()
    }

    async fn run(&mut self, p: Parser<'_>) -> Result<()> {
        log::trace!("i915 run start");
        let values = Values::from_parser(p).await?;
        self.quiet = values.quiet;
        if let Some(cards) = values.ids {
            let min = values.min.map(|v| v.as_megahertz().trunc() as u64);
            let max = values.max.map(|v| v.as_megahertz().trunc() as u64);
            let boost = values.boost.map(|v| v.as_megahertz().trunc() as u64);
            for id in cards {
                if let Some(v) = min {
                    syx::i915::set_min_freq_mhz(id, v).await?;
                }
                if let Some(v) = max {
                    syx::i915::set_max_freq_mhz(id, v).await?;
                }
                if let Some(v) = boost {
                    syx::i915::set_boost_freq_mhz(id, v).await?;
                }
            }
        }
        log::trace!("i915 run done");
        Ok(())
    }

    async fn summary(&self) -> Vec<Formatter> {
        if self.quiet.is_none() { summary::summary().await } else { vec![] }
    }

    fn default_summary(&self) -> bool {
        false
    }
}
