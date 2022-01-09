mod args;
mod table;

use measurements::Frequency;
use syx::drm::Cache as DrmCard;
use tokio::task::JoinHandle;

use crate::cli::Arg;
use crate::Result;

#[derive(Debug)]
pub(crate) struct I915 {
    pub(crate) i915: Option<Vec<u64>>,
    pub(crate) i915_min: Option<Frequency>,
    pub(crate) i915_max: Option<Frequency>,
    pub(crate) i915_boost: Option<Frequency>,
}

impl I915 {
    pub(crate) fn args() -> impl IntoIterator<Item = Arg> {
        args::args()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.i915.as_ref().map(|v| v.is_empty()).unwrap_or(true)
            || (self.i915_min.is_none() && self.i915_max.is_none() && self.i915_boost.is_none())
    }

    pub(crate) async fn apply(&self) -> Result<()> {
        log::trace!("i915 apply start");
        if let Some(i915) = self.i915.clone() {
            for id in i915 {
                if let Some(v) = self.i915_min {
                    let v = v.as_megahertz().trunc() as u64;
                    syx::i915::set_min_freq_mhz(id, v).await?;
                }
                if let Some(v) = self.i915_max {
                    let v = v.as_megahertz().trunc() as u64;
                    syx::i915::set_max_freq_mhz(id, v).await?;
                }
                if let Some(v) = self.i915_boost {
                    let v = v.as_megahertz().trunc() as u64;
                    syx::i915::set_boost_freq_mhz(id, v).await?;
                }
            }
        }
        log::trace!("i915 apply done");
        Ok(())
    }

    pub(crate) async fn tabulate(drm_cards: Vec<DrmCard>) -> Vec<JoinHandle<Option<String>>> {
        table::tabulate(drm_cards).await
    }
}
