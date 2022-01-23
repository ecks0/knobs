use crate::Result;

pub(super) async fn run(values: super::Values) -> Result<()> {
    log::trace!("rapl run start");
    if let Some(constraint_ids) = values.constraint_ids {
        let limit = values.limit.map(|v| v.as_microwatts().trunc() as u64);
        let window = values.window.map(|v| v.as_micros().try_into().unwrap());
        for constraint in constraint_ids.constraints {
            let id = (constraint_ids.package, constraint_ids.subzone, constraint);
            if let Some(v) = limit {
                syx::intel_rapl::constraint::set_power_limit_uw(id, v).await?;
            }
            if let Some(v) = window {
                syx::intel_rapl::constraint::set_time_window_us(id, v).await?;
            }
        }
    }
    log::trace!("rapl run done");
    Ok(())
}
