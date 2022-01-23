use std::collections::HashSet;
use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use futures::stream::{self, StreamExt as _, TryStreamExt as _};

use crate::app::parser::Integer as _;
use crate::util::once;
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
struct DrmId<T>(u64, PhantomData<T>)
where
    T: DrmDriver;

impl<T> DrmId<T>
where
    T: DrmDriver,
{
    async fn from_card_id(v: CardId) -> Result<Self> {
        let cards = once::drm_cards().await;
        let card = match v.clone() {
            CardId::BusId(v) => {
                let v = syx::BusId::from(v);
                let mut card = None;
                for c in cards {
                    if v == c.bus_id().await? {
                        card = Some(c);
                        break;
                    }
                }
                if let Some(card) = card {
                    Ok(card)
                } else {
                    Err(Error::parse_value(format!(
                        "drm card not found on the system: {:?}",
                        v
                    )))
                }
            },
            CardId::Index(v) => {
                if let Some(card) = cards.into_iter().find(|card| v == card.id()) {
                    Ok(card)
                } else {
                    Err(Error::parse_value(format!(
                        "drm card not found on the system: {}",
                        v
                    )))
                }
            },
        }?;
        let wanted = T::driver();
        let found = card.driver().await?;
        let r = if wanted == found.as_str() {
            Ok(card.id())
        } else {
            Err(Error::parse_value(format!(
                "drm card {}: expected driver {:?} but system reports {:?}",
                v, wanted, found
            )))
        }?;
        Ok(Self(r, PhantomData))
    }

    async fn from_str(v: &str) -> Result<Self> {
        let r = CardId::from_str(v)?;
        let r = Self::from_card_id(r).await?;
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

#[derive(Debug)]
pub(super) struct DrmIds<T>(Vec<u64>, PhantomData<T>)
where
    T: DrmDriver;

impl<T> DrmIds<T>
where
    T: DrmDriver,
{
    pub(super) async fn from_str(s: &str) -> Result<Self> {
        log::trace!("parse drm ids start");
        let mut v: Vec<_> = stream::iter(s.split(','))
            .then(DrmId::<T>::from_str)
            .try_fold(HashSet::new(), |mut set, v| async move {
                set.insert(u64::from(v));
                Ok(set)
            })
            .await?
            .into_iter()
            .collect();
        v.sort_unstable();
        let r = Self(v, PhantomData);
        log::trace!("parse drm ids done");
        Ok(r)
    }
}

impl<T> IntoIterator for DrmIds<T>
where
    T: DrmDriver,
{
    type IntoIter = std::vec::IntoIter<u64>;
    type Item = u64;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
