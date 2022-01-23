use crate::Result;

pub(super) async fn run(values: super::Values) -> Result<()> {
    log::trace!("i915 run start");
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
