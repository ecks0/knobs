use std::marker::PhantomData;
use std::str::FromStr;

use async_trait::async_trait;

use crate::util::convert::{AsyncFromStr, AsyncTryFrom};
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
            None => Err(Error::parse_value("Invalid syntax for bus id")),
        }
    }
}

impl From<BusId> for syx::BusId {
    fn from(v: BusId) -> Self {
        let (bus, id) = (v.bus, v.id);
        syx::BusId { bus, id }
    }
}

#[derive(Debug)]
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
                let r = v
                    .parse::<u64>()
                    .map_err(|_| Error::parse_value("Invalid syntax for card index"))?;
                Self::Index(r)
            },
        };
        Ok(r)
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
impl<T> AsyncTryFrom<CardId> for DrmId<T>
where
    T: DrmDriver,
{
    type Error = Error;

    async fn async_try_from(v: CardId) -> Result<Self> {
        let v = match v {
            CardId::BusId(b) => {
                let b = b.into();
                let r = syx::drm::index(&b).await?;
                Ok(r)
            },
            CardId::Index(v) => {
                if syx::drm::exists(v).await? {
                    Ok(v)
                } else {
                    Err(Error::parse_value(format!("Drm card {} not found", v)))
                }
            },
        }?;
        let wanted = T::driver();
        let found = syx::drm::driver(v).await?;
        let r = if wanted == found.as_str() {
            Ok(v)
        } else {
            Err(Error::parse_value(format!(
                "Drm card {}: expected {} driver, found {}",
                v, wanted, found
            )))
        }?;
        Ok(Self(r, PhantomData))
    }
}

#[async_trait]
impl<T> AsyncFromStr for DrmId<T>
where
    T: DrmDriver,
{
    type Error = Error;

    async fn async_from_str(v: &str) -> Result<Self> {
        let r = CardId::from_str(v)?;
        let r = Self::async_try_from(r).await?;
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
