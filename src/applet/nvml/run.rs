use crate::Result;

pub(super) async fn run(values: super::Values) -> Result<()> {
    log::trace!("nvml run start");
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
    log::trace!("nvml run done");
    Ok(())
}
