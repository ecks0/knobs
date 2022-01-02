mod args;
mod table;

use futures::Future;
use measurements::{Frequency, Power};

use crate::cli::Arg;
use crate::Result;

#[derive(Debug)]
pub(crate) struct Nvml {
    pub(crate) nvml: Option<Vec<u64>>,
    pub(crate) nvml_gpu_min: Option<Frequency>,
    pub(crate) nvml_gpu_max: Option<Frequency>,
    pub(crate) nvml_power: Option<Power>,
}

impl Nvml {
    pub(crate) fn args() -> impl IntoIterator<Item = Arg> {
        args::args()
    }

    #[rustfmt::skip]
    pub(crate) fn is_empty(&self) -> bool {
        self.nvml.as_ref().map(|v| v.is_empty()).unwrap_or(true) || (
            self.nvml_gpu_min.is_none() &&
            self.nvml_gpu_max.is_none() &&
            self.nvml_power.is_none()
        )
    }

    pub(crate) async fn apply(&self) -> Result<()> {
        if let Some(nvml) = self.nvml.clone() {
            for id in nvml {
                if let Some(min) = self.nvml_gpu_min {
                    if let Some(max) = self.nvml_gpu_max {
                        let min = min.as_megahertz().trunc() as u32;
                        let max = max.as_megahertz().trunc() as u32;
                        syx::nvml::set_gfx_freq(id, min, max).await?;
                    }
                    if let Some(v) = self.nvml_power {
                        let v = v.as_milliwatts().trunc() as u32;
                        syx::nvml::set_power_limit(id, v).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn tabulate() -> impl Future<Output = Option<String>> {
        table::tabulate()
    }
}
