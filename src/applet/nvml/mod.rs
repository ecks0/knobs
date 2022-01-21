mod args;
mod summary;

use async_trait::async_trait;
use measurements::{Frequency, Power};

use crate::applet::{Applet, Formatter};
use crate::cli::{Arg, Parser};
use crate::Result;

#[derive(Debug)]
struct Values {
    cards: Option<Vec<u64>>,
    gpu_min: Option<Frequency>,
    gpu_max: Option<Frequency>,
    gpu_reset: Option<()>,
    power: Option<Power>,
    power_reset: Option<()>,
    quiet: Option<()>,
}

#[derive(Debug, Default)]
pub(crate) struct Nvml {
    quiet: Option<()>,
}

#[async_trait]
impl Applet for Nvml {
    fn name(&self) -> &'static str {
        "nvml"
    }

    fn bin(&self) -> Option<&'static str> {
        Some("knvml")
    }

    fn about(&self) -> &'static str {
        "View or set nvml values"
    }

    fn args(&self) -> Vec<Arg> {
        args::args()
    }

    async fn run(&mut self, p: Parser<'_>) -> Result<()> {
        log::trace!("nvml apply start");
        let values = Values::from_parser(p).await?;
        self.quiet = values.quiet;
        if let Some(nvml) = values.cards {
            let gpu_min = values.gpu_min.map(|v| v.as_megahertz().trunc() as u32);
            let gpu_max = values.gpu_max.map(|v| v.as_megahertz().trunc() as u32);
            let power = values.power.map(|v| v.as_milliwatts().trunc() as u32);
            for id in nvml {
                if let Some(min) = gpu_min {
                    if let Some(max) = gpu_max {
                        syx::nvml::set_gfx_freq(id, min, max).await?;
                    }
                }
                if values.gpu_reset.is_some() {
                    syx::nvml::reset_gfx_freq(id).await?;
                }
                if let Some(v) = power {
                    syx::nvml::set_power_limit(id, v).await?;
                }
                if values.power_reset.is_some() {
                    syx::nvml::reset_power_limit(id).await?;
                }
            }
        }
        log::trace!("nvml apply done");
        Ok(())
    }

    async fn summary(&self) -> Vec<Formatter> {
        if self.quiet.is_none() { summary::summary().await } else { vec![] }
    }

    fn default_summary(&self) -> bool {
        false
    }
}
