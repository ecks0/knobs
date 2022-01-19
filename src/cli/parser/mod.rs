mod bool;
mod cpu;
mod drm;
mod frequency;
mod number;
mod power;
mod rapl;
mod time;

use std::str::FromStr;
use std::time::Duration;

use measurements::{Frequency, Power};

use crate::applet::rapl::ConstraintIds as RaplConstraintIds;
pub(crate) use crate::cli::parser::drm::{DrmDriver, I915Driver, NvmlDriver};
use crate::cli::parser::number::Integer;
use crate::cli::parser::power::Watts;
use crate::{Error, Result};

#[derive(Debug)]
pub(crate) struct Parser<'a>(&'a clap::ArgMatches);

impl<'a> From<&'a clap::ArgMatches> for Parser<'a> {
    fn from(v: &'a clap::ArgMatches) -> Self {
        Self(v)
    }
}

impl<'a> Parser<'a> {
    pub(crate) fn bool(&self, name: &str) -> Result<Option<bool>> {
        self.str(name)
            .map(bool::Bool::from_str)
            .transpose()
            .map(|v| v.map(Into::into))
            .map_err(|e| Error::parse_flag(e, name))
    }

    pub(crate) async fn cpu_ids(&self, name: &str) -> Result<Option<Vec<u64>>> {
        if let Some(v) = self.str(name) {
            cpu::CpuIds::from_str(v)
                .await
                .map(|v| Some(v.into_iter().collect()))
                .map_err(|e| Error::parse_flag(e, name))
        } else {
            Ok(None)
        }
    }

    pub(crate) async fn drm_ids<T>(&self, name: &str) -> Result<Option<Vec<u64>>>
    where
        T: DrmDriver,
    {
        if let Some(v) = self.str(name) {
            drm::DrmIds::<T>::from_str(v)
                .await
                .map(|v| Some(v.into_iter().collect()))
                .map_err(|e| Error::parse_flag(e, name))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn flag(&self, name: &str) -> Option<()> {
        if self.0.is_present(name) { Some(()) } else { None }
    }

    pub(crate) fn int<I: Integer>(&self, name: &str) -> Result<Option<I>> {
        self.str(name).map(I::parse).transpose().map_err(|e| Error::parse_flag(e, name))
    }

    pub(crate) fn megahertz(&self, name: &str) -> Result<Option<Frequency>> {
        self.str(name)
            .map(frequency::Megahertz::from_str)
            .transpose()
            .map(|v| v.map(Into::into))
            .map_err(|e| Error::parse_flag(e, name))
    }

    pub(crate) fn microseconds(&self, name: &str) -> Result<Option<Duration>> {
        self.str(name)
            .map(time::Microseconds::from_str)
            .transpose()
            .map(|v| v.map(Into::into))
            .map_err(|e| Error::parse_flag(e, name))
    }

    pub(crate) async fn rapl_constraint_ids(
        &self,
        package: &str,
        subzone: &str,
        constraint: &str,
    ) -> Result<Option<RaplConstraintIds>> {
        if let Some(p) = self.int(package)? {
            if let Some(constraints) = self.str(constraint) {
                let s = self.int::<u64>(subzone).map_err(|e| Error::parse_flag(e, subzone))?;
                let c: Vec<_> = constraints
                    .split(',')
                    .map(u64::parse)
                    .collect::<Result<_>>()
                    .map_err(|e| Error::parse_flag(e, constraints))?;
                let r = rapl::constraint_ids(p, s, c).await.map_err(|e| {
                    Error::parse_flag(e, format!("{}/--{}/--{}", package, subzone, constraints))
                })?;
                return Ok(Some(r));
            }
        }
        Ok(None)
    }

    pub(crate) fn str(&self, name: &str) -> Option<&str> {
        self.0.value_of(name)
    }

    pub(crate) fn string(&self, name: &str) -> Option<String> {
        self.str(name).map(String::from)
    }

    pub(crate) fn watts(&self, name: &str) -> Result<Option<Power>> {
        self.str(name)
            .map(Watts::from_str)
            .transpose()
            .map(|v| v.map(Into::into))
            .map_err(|e| Error::parse_flag(e, name))
    }
}
