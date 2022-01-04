use std::fmt::Display;
use std::str::FromStr;

use async_trait::async_trait;

use crate::cli::parser::number::Integer;
use crate::util::convert::{AsyncFromStr, AsyncTryFrom};
use crate::util::cpu;
use crate::{Error, Result};

#[derive(Debug)]
enum Range<T> {
    From(std::ops::RangeFrom<T>),
    Inclusive(std::ops::RangeInclusive<T>),
    ToInclusive(std::ops::RangeToInclusive<T>),
    Unbounded,
}

impl<T> FromStr for Range<T>
where
    T: Integer,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.split("..").collect::<Vec<_>>()[..] {
            [idx] => {
                let r = T::parse(idx)?;
                let r = r..=r;
                let r = Self::Inclusive(r);
                Ok(r)
            },
            [start, end] => {
                let r = if start.is_empty() && end.is_empty() {
                    Self::Unbounded
                } else if start.is_empty() {
                    let end = T::parse(end)?;
                    let r = ..=end;
                    Self::ToInclusive(r)
                } else if end.is_empty() {
                    let start = T::parse(start)?;
                    let r = start..;
                    Self::From(r)
                } else {
                    let start = T::parse(start)?;
                    let end = T::parse(end)?;
                    let r = if start <= end { start..=end } else { end..=start };
                    Self::Inclusive(r)
                };
                Ok(r)
            },
            _ => Err(Error::parse_value(format!(
                "could not parse as range: {}",
                s
            ))),
        }
    }
}

impl<T> Display for Range<T>
where
    T: Integer + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match &self {
            Self::From(r) => format!("{}..", r.start),
            Self::Inclusive(r) => format!("{}..{}", r.start(), r.end()),
            Self::ToInclusive(r) => format!("..{}", r.end),
            Self::Unbounded => "..".to_string(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub(super) struct CpuIds(std::ops::RangeInclusive<u64>);

#[async_trait]
impl AsyncTryFrom<Range<u64>> for CpuIds {
    type Error = Error;

    async fn async_try_from(v: Range<u64>) -> Result<Self> {
        fn err(v: &Range<u64>) -> Error {
            Error::parse_value(format!(
                "range includes cpu ids not found on the system: {}",
                v
            ))
        }
        let all = cpu::ids().await;
        let cpu_0 = all.iter().min().cloned();
        let cpu_n = all.iter().max().cloned();
        if let (Some(cpu_0), Some(cpu_n)) = (cpu_0, cpu_n) {
            let r = match &v {
                Range::From(r) => {
                    if r.start <= cpu_n {
                        Ok(r.start..=cpu_n)
                    } else {
                        Err(err(&v))
                    }
                },
                Range::Inclusive(r) => {
                    if r.start() <= &cpu_n && r.end() <= &cpu_n {
                        Ok(r.clone())
                    } else {
                        Err(err(&v))
                    }
                },
                Range::ToInclusive(r) => {
                    if r.end <= cpu_n {
                        Ok(cpu_0..=r.end)
                    } else {
                        Err(err(&v))
                    }
                },
                Range::Unbounded => Ok(cpu_0..=cpu_n),
            }?;
            r.clone()
                .into_iter()
                .try_for_each(|r| if all.contains(&r) { Ok(()) } else { Err(err(&v)) })?;
            Ok(Self(r))
        } else {
            Err(Error::parse_value(
                "unable to read system cpu ids for argument validation",
            ))
        }
    }
}

#[async_trait]
impl AsyncFromStr for CpuIds {
    type Error = Error;

    async fn async_from_str(v: &str) -> Result<Self> {
        let r = Range::from_str(v)?;
        let r = Self::async_try_from(r).await?;
        Ok(r)
    }
}

impl IntoIterator for CpuIds {
    type IntoIter = std::ops::RangeInclusive<u64>;
    type Item = u64;

    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}
