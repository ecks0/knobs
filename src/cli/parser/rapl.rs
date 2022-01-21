use crate::applet::RaplConstraintIds;
use crate::{Error, Result};

pub(super) async fn constraint_ids(
    package: u64,
    subzone: Option<u64>,
    constraints: Vec<u64>,
) -> Result<RaplConstraintIds> {
    if !syx::intel_rapl::zone::exists((package, subzone)).await? {
        let mut s = format!("package {} ", package);
        if let Some(subzone) = subzone {
            s.push_str(&format!("subzone {} ", subzone));
        }
        s.push_str("not found");
        return Err(Error::parse_value(s));
    }
    for constraint in constraints.clone() {
        if !syx::intel_rapl::constraint::exists((package, subzone, constraint)).await? {
            let mut s = format!("constraint {} not found in package {}", constraint, package);
            if let Some(subzone) = subzone {
                s.push_str(&format!(" subzone {} ", subzone));
            }
            return Err(Error::parse_value(s));
        }
    }
    let r = RaplConstraintIds {
        package,
        subzone,
        constraints,
    };
    Ok(r)
}
