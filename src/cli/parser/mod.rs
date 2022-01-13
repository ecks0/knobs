mod bool;
mod cpu;
mod drm;
mod frequency;
mod number;
mod power;
mod time;

use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;

use futures::stream::{self, TryStreamExt as _};
use measurements::{Frequency, Power};

use crate::cli::parser::bool::Bool;
use crate::cli::parser::cpu::CpuIds;
use crate::cli::parser::drm::DrmId;
pub(crate) use crate::cli::parser::drm::{DrmDriver, I915Driver, NvmlDriver};
use crate::cli::parser::frequency::Megahertz;
use crate::cli::parser::number::Integer;
use crate::cli::parser::power::Watts;
use crate::cli::parser::time::Microseconds;
use crate::cli::{Arg, ARGV0};
use crate::util::convert::*;
use crate::*;

impl<'a> From<&'a Arg> for clap::Arg<'a> {
    fn from(v: &'a Arg) -> Self {
        let name = v.name.expect("Cli argument name name is missing");
        let mut a = clap::Arg::new(name);
        if let Some(long) = v.long {
            a = a.long(long);
        }
        if let Some(short) = v.short {
            a = a.short(short);
        }
        if let Some(value_name) = v.value_name {
            a = a.takes_value(true).value_name(value_name);
        }
        if let Some(help) = &v.help {
            a = a.help(help.as_str());
        }
        if let Some(help_long) = &v.help_long {
            a = a.long_help(help_long.as_str());
        }
        if let Some(requires) = &v.requires {
            for required in requires {
                a = a.requires(required);
            }
        }
        if let Some(conflicts) = &v.conflicts {
            for conflicted in conflicts {
                a = a.conflicts_with(conflicted);
            }
        }
        if let Some(raw) = v.raw {
            a = a.raw(raw);
        }
        a
    }
}

#[derive(Debug)]
pub(crate) struct Parser {
    pub(super) matches: clap::ArgMatches,
}

impl Parser {
    pub(super) fn new(args: &[Arg], argv: &[&str]) -> Result<Self> {
        let clap_args: Vec<clap::Arg> = args.iter().map(From::from).collect();
        let matches = clap::App::new(ARGV0)
            .color(clap::ColorChoice::Never)
            .setting(clap::AppSettings::DeriveDisplayOrder)
            .setting(clap::AppSettings::DisableHelpSubcommand)
            .setting(clap::AppSettings::TrailingVarArg)
            .version(clap::crate_version!())
            .args(&clap_args)
            .try_get_matches_from(argv)
            .map_err(Error::Clap)?;
        let r = Self { matches };
        Ok(r)
    }

    pub(crate) fn bool(&self, name: &str) -> Result<Option<bool>> {
        Ok(self
            .str(name)
            .map(Bool::from_str)
            .transpose()
            .map_err(|e| Error::parse_flag(e, name))?
            .map(Into::into))
    }

    pub(crate) async fn cpu_ids(&self, name: &str) -> Result<Option<Vec<u64>>> {
        Ok(if let Some(v) = self.str(name) {
            let i = v.split(',').map(Result::Ok);
            let mut r: Vec<_> = stream::iter(i)
                .map_ok(CpuIds::from_str_ref)
                .try_fold(HashSet::new(), |mut set, v| async move {
                    let v = v.await?;
                    set.extend(v);
                    Ok(set)
                })
                .await
                .map_err(|e| Error::parse_flag(e, name))?
                .into_iter()
                .collect();
            r.sort_unstable();
            Some(r)
        } else {
            None
        })
    }

    pub(crate) async fn drm_ids<T>(&self, name: &str) -> Result<Option<Vec<u64>>>
    where
        T: DrmDriver,
    {
        Ok(if let Some(v) = self.str(name) {
            let i = v.split(',').map(Result::Ok);
            let mut r: Vec<_> = stream::iter(i)
                .and_then(|v| async move {
                    let r: u64 = DrmId::<T>::from_str_ref(v).await?.into();
                    Ok(r)
                })
                .try_collect()
                .await
                .map_err(|e| Error::parse_flag(e, name))?;
            r.sort_unstable();
            r.dedup();
            Some(r)
        } else {
            None
        })
    }

    pub(crate) fn flag(&self, name: &str) -> Option<()> {
        if self.matches.is_present(name) { Some(()) } else { None }
    }

    pub(crate) fn int<I: Integer>(&self, name: &str) -> Result<Option<I>> {
        self.str(name).map(I::parse).transpose().map_err(|e| Error::parse_flag(e, name))
    }

    pub(crate) fn megahertz(&self, name: &str) -> Result<Option<Frequency>> {
        Ok(self
            .str(name)
            .map(Megahertz::from_str)
            .transpose()
            .map_err(|e| Error::parse_flag(e, name))?
            .map(Into::into))
    }

    pub(crate) fn microseconds(&self, name: &str) -> Result<Option<Duration>> {
        Ok(self
            .str(name)
            .map(Microseconds::from_str)
            .transpose()
            .map_err(|e| Error::parse_flag(e, name))?
            .map(Into::into))
    }

    pub(crate) async fn rapl_constraint(
        &self,
        package_name: &str,
        subzone_name: &str,
        constraint_name: &str,
    ) -> Result<Option<(u64, Option<u64>, u64)>> {
        // FIXME decompose
        if let Some(package) = self.int::<u64>(package_name)? {
            if let Some(constraint) = self.int::<u64>(constraint_name)? {
                let subzone = self.int::<u64>(subzone_name)?;
                if !syx::intel_rapl::zone::exists((package, None))
                    .await
                    .map_err(|e| Error::parse_flag(e.into(), package_name))?
                {
                    let s = format!("Package not found: {}", package);
                    return Err(Error::parse_flag(Error::parse_value(s), package_name));
                }
                if let Some(subzone) = subzone {
                    if !syx::intel_rapl::zone::exists((package, subzone))
                        .await
                        .map_err(|e| Error::parse_flag(e.into(), subzone_name))?
                    {
                        let s = format!("Subzone {} not found in package {}", subzone, package);
                        return Err(Error::parse_flag(Error::parse_value(s), subzone_name));
                    }
                }
                let id = (package, subzone, constraint);
                if !syx::intel_rapl::constraint::exists(id).await? {
                    let mut s = format!(
                        "Constraint {} not found for package {}",
                        constraint, package
                    );
                    if let Some(v) = subzone {
                        s.push_str(&format!(", subzone {}", v));
                    }
                    return Err(Error::parse_flag(Error::parse_value(s), constraint_name));
                }
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    pub(crate) fn str(&self, name: &str) -> Option<&str> {
        self.matches.value_of(name)
    }

    pub(crate) fn string(&self, name: &str) -> Option<String> {
        self.str(name).map(String::from)
    }

    pub(crate) fn watts(&self, name: &str) -> Result<Option<Power>> {
        Ok(self
            .str(name)
            .map(Watts::from_str)
            .transpose()
            .map_err(|e| Error::parse_flag(e, name))?
            .map(Into::into))
    }
}
