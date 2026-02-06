use crate::{Amount, Level, Price, RkyvIndexMapDecimal};
use core::fmt;
use core::str::FromStr;
use derive_more::{AsRef, Deref, DerefMut, Into};
use indexmap::IndexMap;
use polymarket_client_sdk::clob::types::response::OrderSummary;
use rkyv::Archive;
use rustc_hash::FxBuildHasher;
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// The orderbook is represented as two `BookSide` (`bids` and `asks`) because some APIs may return a crossed book during fast moves (max bid price ≥ min ask price).
#[derive(Archive, PartialEq, Eq, Clone, Debug, Deref, DerefMut, AsRef, Into)]
pub struct BookSideMap(#[rkyv(with = RkyvIndexMapDecimal)] IndexMap<Price, Amount, FxBuildHasher>);

impl Serialize for BookSideMap {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer
            .serialize_map(Some(self.len()))
            .and_then(|mut map| {
                let result = self.iter().try_for_each(|(price, size)| {
                    let price_string = price.to_string();
                    let size_string = size.to_string();
                    map.serialize_entry(&price_string, &size_string)
                });
                result.and_then(|()| map.end())
            })
    }
}

impl<'de> Deserialize<'de> for BookSideMap {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BookSideMapVisitor;

        impl<'de> Visitor<'de> for BookSideMapVisitor {
            type Value = BookSideMap;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map of decimal prices to decimal sizes")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut map = IndexMap::with_capacity_and_hasher(access.size_hint().unwrap_or(0), FxBuildHasher);
                let mut entries = core::iter::from_fn(|| match access.next_entry::<String, String>() {
                    Ok(Some(entry)) => Some(Ok(entry)),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                });
                let result = entries.try_for_each(|entry| {
                    let (price_string, size_string) = match entry {
                        Ok(entry) => entry,
                        Err(error) => return Err(error),
                    };
                    let price = match Price::from_str(&price_string) {
                        Ok(price) => price,
                        Err(error) => return Err(A::Error::custom(error)),
                    };
                    let size = match Amount::from_str(&size_string) {
                        Ok(size) => size,
                        Err(error) => return Err(A::Error::custom(error)),
                    };
                    match map.get(&price) {
                        Some(existing_size) if *existing_size != size => Err(A::Error::custom("price level conflicts with existing size")),
                        Some(_) => Ok(()),
                        None => {
                            map.insert(price, size);
                            Ok(())
                        }
                    }
                });
                match result {
                    Ok(()) => Ok(BookSideMap(map)),
                    Err(error) => Err(error),
                }
            }
        }

        deserializer.deserialize_map(BookSideMapVisitor)
    }
}

impl BookSideMap {
    pub fn new(map: impl Into<IndexMap<Price, Amount, FxBuildHasher>>) -> Self {
        Self(map.into())
    }

    pub fn set(&mut self, map: impl Into<IndexMap<Price, Amount, FxBuildHasher>>) {
        self.0 = map.into();
    }

    pub fn into_inner(self) -> IndexMap<Price, Amount, FxBuildHasher> {
        self.0
    }

    pub fn min_price(&self) -> Option<&Price> {
        self.keys().min()
    }

    pub fn max_price(&self) -> Option<&Price> {
        self.keys().max()
    }

    pub fn min(&self) -> Option<Level> {
        self.iter().min_by_key(|x| x.0).map(Level::from)
    }

    pub fn max(&self) -> Option<Level> {
        self.iter().max_by_key(|x| x.0).map(Level::from)
    }

    /// Expected invocation form: `bids.crosses_up(asks)`
    pub fn crosses_up(&self, other: &BookSideMap) -> bool {
        let self_max_price = self.max_price();
        let other_min_price = other.min_price();
        match (self_max_price, other_min_price) {
            (Some(max_price), Some(min_price)) => max_price >= min_price,
            // can't cross if no prices (no orders)
            _ => false,
        }
    }
}

impl From<IndexMap<Price, Amount, FxBuildHasher>> for BookSideMap {
    fn from(value: IndexMap<Price, Amount, FxBuildHasher>) -> Self {
        Self(value)
    }
}

impl From<&IndexMap<Price, Amount, FxBuildHasher>> for BookSideMap {
    fn from(value: &IndexMap<Price, Amount, FxBuildHasher>) -> Self {
        Self(value.clone())
    }
}

impl TryFrom<Vec<OrderSummary>> for BookSideMap {
    type Error = ConvertVecOrderSummaryToBookSideError;

    fn try_from(summaries: Vec<OrderSummary>) -> Result<Self, Self::Error> {
        use ConvertVecOrderSummaryToBookSideError::*;
        let map = IndexMap::with_capacity_and_hasher(summaries.len(), FxBuildHasher);
        let result = summaries.into_iter().try_fold(map, |mut map, summary| {
            let OrderSummary {
                price,
                size,
                ..
            } = summary;
            match map.get(&price) {
                Some(existing_size) if *existing_size != size => Err(PriceLevelConflicts {
                    price,
                    existing_size: *existing_size,
                    incoming_size: size,
                }),
                Some(_) => Ok(map),
                None => {
                    map.insert(price, size);
                    Ok(map)
                }
            }
        });
        match result {
            Ok(map) => Ok(Self(map)),
            Err(error) => Err(error),
        }
    }
}

#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConvertVecOrderSummaryToBookSideError {
    #[error("price level '{price}' conflicts with sizes '{existing_size}' and '{incoming_size}'")]
    PriceLevelConflicts { price: Price, existing_size: Amount, incoming_size: Amount },
}

impl From<BookSideMap> for Vec<OrderSummary> {
    fn from(value: BookSideMap) -> Self {
        let BookSideMap(map) = value;
        map.into_iter()
            .map(|(price, size)| OrderSummary::builder().price(price).size(size).build())
            .collect()
    }
}
