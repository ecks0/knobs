use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use async_trait::async_trait;

use crate::cli::parser::Integer as _;
use crate::util::convert::{FromStrRef, TryFromValue};
use crate::{Error, Result};

#[derive(Clone, Debug)]
struct BusId {
    bus: String,
    id: String,
}

impl FromStr for BusId {
    type Err = Error;

    fn from_str(v: &str) -> Result<Self> {
        match v.split_once(':') {
            Some((bus, id)) => {
                let (bus, id) = (bus.into(), id.into());
                let r = Self { bus, id };
                Ok(r)
            },
            None => Err(Error::parse_value(format!(
                "expected bus id syntax {:?}, got {:?}",
                "BUS:ID", v
            ))),
        }
    }
}

impl Display for BusId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.bus, self.id)
    }
}

impl From<BusId> for syx::BusId {
    fn from(v: BusId) -> Self {
        let (bus, id) = (v.bus, v.id);
        syx::BusId { bus, id }
    }
}

#[derive(Clone, Debug)]
enum CardId {
    BusId(BusId),
    Index(u64),
}

impl FromStr for CardId {
    type Err = Error;

    fn from_str(v: &str) -> Result<Self> {
        let r = match v.contains(':') {
            true => {
                let r = BusId::from_str(v)?;
                Self::BusId(r)
            },
            false => {
                let r = u64::parse(v)?;
                Self::Index(r)
            },
        };
        Ok(r)
    }
}

impl Display for CardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::BusId(v) => v.to_string(),
            Self::Index(v) => v.to_string(),
        };
        write!(f, "{}", s)
    }
}

pub(crate) trait DrmDriver {
    fn driver() -> &'static str;
}

#[derive(Debug)]
pub(crate) struct I915Driver;

impl DrmDriver for I915Driver {
    fn driver() -> &'static str {
        "i915"
    }
}

#[derive(Debug)]
pub(crate) struct NvmlDriver;

impl DrmDriver for NvmlDriver {
    fn driver() -> &'static str {
        "nvidia"
    }
}

#[derive(Debug)]
pub(super) struct DrmId<T>(u64, PhantomData<T>)
where
    T: DrmDriver;

#[async_trait]
impl<T> TryFromValue<CardId> for DrmId<T>
where
    T: DrmDriver,
{
    type Error = Error;

    async fn try_from_value(v: CardId) -> Result<Self> {
        let index = match v.clone() {
            CardId::BusId(v) => {
                let v = syx::BusId::from(v);
                let v = syx::drm::index(&v).await?;
                Ok(v)
            },
            CardId::Index(v) => {
                if syx::drm::exists(v).await? {
                    Ok(v)
                } else {
                    Err(Error::parse_value(format!(
                        "drm card not found on the system: {}",
                        v
                    )))
                }
            },
        }?;
        let wanted = T::driver();
        let found = syx::drm::driver(index).await?;
        let r = if wanted == found.as_str() {
            Ok(index)
        } else {
            Err(Error::parse_value(format!(
                "drm card {}: expected driver {:?} but system reports {:?}",
                v, wanted, found
            )))
        }?;
        Ok(Self(r, PhantomData))
    }
}

#[async_trait]
impl<T> FromStrRef for DrmId<T>
where
    T: DrmDriver,
{
    type Error = Error;

    async fn from_str_ref(v: &str) -> Result<Self> {
        let r = CardId::from_str(v)?;
        let r = Self::try_from_value(r).await?;
        Ok(r)
    }
}

impl<T> From<DrmId<T>> for u64
where
    T: DrmDriver,
{
    fn from(v: DrmId<T>) -> Self {
        v.0
    }
}
