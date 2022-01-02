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

use async_trait::async_trait;
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
use crate::cli::Arg;
use crate::util::convert::*;
use crate::{Cpu, Error, Nvml, Profile, Profiles, Rapl, Result, I915, NAME};

const ARGS: &str = "ARGS";

impl<'a> From<&'a Arg> for clap::Arg<'a, 'a> {
    fn from(v: &'a Arg) -> Self {
        let mut a = clap::Arg::with_name(v.name);
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
            a = a.help(help);
        }
        if let Some(help_long) = &v.help_long {
            a = a.long_help(help_long);
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
        a
    }
}

#[derive(Debug)]
pub(crate) struct Parser<'a> {
    args: &'a [Arg],
    matches: clap::ArgMatches<'a>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(args: &'a [Arg], argv: &[String]) -> Result<Self> {
        let clap_args: Vec<clap::Arg> = args.iter().map(From::from).collect();
        let matches = clap::App::new(NAME)
            .setting(clap::AppSettings::DeriveDisplayOrder)
            .setting(clap::AppSettings::DisableHelpSubcommand)
            .setting(clap::AppSettings::DisableVersion)
            .setting(clap::AppSettings::TrailingVarArg)
            .setting(clap::AppSettings::UnifiedHelpMessage)
            .version(clap::crate_version!())
            .args(&clap_args)
            .arg(clap::Arg::with_name(ARGS).raw(true))
            .get_matches_from_safe(argv)
            .map_err(Error::Clap)?;
        let r = Self { args, matches };
        Ok(r)
    }

    pub(crate) fn bool(&self, name: &str) -> Result<Option<bool>> {
        Ok(self
            .str(name)
            .map(Bool::from_str)
            .transpose()
            .map_err(|e| Error::parse_flag(name, e))?
            .map(Into::into))
    }

    pub(crate) async fn cpu_ids(&self, name: &str) -> Result<Option<Vec<u64>>> {
        let r = if let Some(v) = self.str(name) {
            let i = v.split(',').map(Result::Ok);
            let mut r: Vec<_> = stream::iter(i)
                .map_ok(CpuIds::async_from_str)
                .try_fold(HashSet::new(), |mut set, v| async move {
                    let v = v.await?;
                    set.extend(v);
                    Ok(set)
                })
                .await
                .map_err(|e| Error::parse_flag(name, e))?
                .into_iter()
                .collect();
            r.sort_unstable();
            Some(r)
        } else {
            None
        };
        Ok(r)
    }

    pub(crate) async fn drm_ids<T>(&self, name: &str) -> Result<Option<Vec<u64>>>
    where
        T: DrmDriver,
    {
        let r = if let Some(v) = self.str(name) {
            let i = v.split(',').map(Result::Ok);
            let mut r: Vec<_> = stream::iter(i)
                .and_then(|v| async move {
                    let r: u64 = DrmId::<T>::async_from_str(v).await?.into();
                    Ok(r)
                })
                .try_collect()
                .await
                .map_err(|e| Error::parse_flag(name, e))?;
            r.sort_unstable();
            r.dedup();
            Some(r)
        } else {
            None
        };
        Ok(r)
    }

    pub(crate) fn flag(&self, name: &str) -> Option<()> {
        if self.matches.is_present(name) {
            Some(())
        } else {
            None
        }
    }

    pub(crate) fn int<I: Integer>(&self, name: &str) -> Result<Option<I>> {
        self.str(name)
            .map(I::parse)
            .transpose()
            .map_err(|e| Error::parse_flag(name, e))
    }

    pub(crate) fn megahertz(&self, name: &str) -> Result<Option<Frequency>> {
        Ok(self
            .str(name)
            .map(Megahertz::from_str)
            .transpose()
            .map_err(|e| Error::parse_flag(name, e))?
            .map(Into::into))
    }

    pub(crate) fn microseconds(&self, name: &str) -> Result<Option<Duration>> {
        Ok(self
            .str(name)
            .map(Microseconds::from_str)
            .transpose()
            .map_err(|e| Error::parse_flag(name, e))?
            .map(Into::into))
    }

    pub(crate) fn pstate_epb(&self, name: &str) -> Result<Option<u64>> {
        self.int(name)?
            .map(|v| {
                if v > 15 {
                    Err(Error::parse_flag(
                        name,
                        "Energy/performance bias must be within 0..=15",
                    ))
                } else {
                    Ok(v)
                }
            })
            .transpose()
    }

    pub(crate) async fn rapl_constraint(
        &self,
        package_name: &str,
        subzone_name: &str,
        constraint_name: &str,
    ) -> Result<Option<(u64, Option<u64>, u64)>> {
        if let Some(package) = self.int::<u64>(package_name)? {
            if let Some(constraint) = self.int::<u64>(constraint_name)? {
                let subzone = self.int::<u64>(subzone_name)?;
                if !syx::rapl::zone::exists((package, None))
                    .await
                    .map_err(|e| Error::parse_flag(package_name, e))?
                {
                    return Err(Error::parse_flag(
                        package_name,
                        format!("Package not found: {}", package),
                    ));
                }
                if let Some(subzone) = subzone {
                    if !syx::rapl::zone::exists((package, subzone))
                        .await
                        .map_err(|e| Error::parse_flag(subzone_name, e))?
                    {
                        return Err(Error::parse_flag(
                            subzone_name,
                            format!("Subzone {} not found in package {}", subzone, package),
                        ));
                    }
                }
                let id = (package, subzone, constraint);
                if !syx::rapl::constraint::exists(id).await? {
                    let mut s = format!(
                        "Constraint {} not found for package {}",
                        constraint, package
                    );
                    if let Some(v) = subzone {
                        s.push_str(&format!(", subzone {}", v));
                    }
                    return Err(Error::parse_flag(constraint_name, s));
                }
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    pub(crate) fn str(&self, name: &str) -> Option<&str> {
        self.matches.value_of(name)
    }

    pub(crate) fn strs(&self, name: &str) -> Option<Vec<&str>> {
        self.matches
            .values_of(name)
            .map(|v| v.into_iter().collect())
    }

    pub(crate) fn string(&self, name: &str) -> Option<String> {
        self.str(name).map(String::from)
    }

    pub(crate) fn strings(&self, name: &str) -> Option<Vec<String>> {
        self.strs(name)
            .map(|v| v.into_iter().map(String::from).collect())
    }

    pub(crate) fn watts(&self, name: &str) -> Result<Option<Power>> {
        Ok(self
            .str(name)
            .map(Watts::from_str)
            .transpose()
            .map_err(|e| Error::parse_flag(name, e))?
            .map(Into::into))
    }
}

#[async_trait]
impl<'a> TryFromRef<Parser<'a>> for Profile {
    type Error = Error;

    async fn try_from_ref(v: &Parser<'a>) -> Result<Self> {
        let r = Self {
            cpu: Cpu::try_from_ref(v).await?,
            i915: I915::try_from_ref(v).await?,
            nvml: Nvml::try_from_ref(v).await?,
            rapl: Rapl::try_from_ref(v).await?,
        };
        Ok(r)
    }
}

#[async_trait]
impl<'a> TryFromRef<Parser<'a>> for Profiles {
    type Error = Error;

    async fn try_from_ref(p: &Parser<'a>) -> Result<Self> {
        let mut profiles = vec![];
        let mut args = p.strings(ARGS);
        profiles.push(p.try_ref_into().await?);
        while let Some(mut a) = args {
            a.insert(0, NAME.to_string());
            let p = Parser::new(p.args, &a)?;
            args = p.strings(ARGS);
            profiles.push(p.try_ref_into().await?);
        }
        let r = Profiles(profiles);
        Ok(r)
    }
}
