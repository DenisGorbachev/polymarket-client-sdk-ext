use crate::{Amount, Level, Price};
use indexmap::IndexMap;
use polymarket_client_sdk::clob::types::response::OrderSummary;
use rustc_hash::FxBuildHasher;
use serde::{Deserialize, Serialize};
use subtype::subtype;
use thiserror::Error;

subtype!(
    /// The orderbook is represented as two `BookSide` (`bids` and `asks`) because some APIs may return a crossed book during fast moves (max bid price ≥ min ask price).
    #[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
    pub struct BookSide(IndexMap<Price, Amount, FxBuildHasher>)
);

impl BookSide {
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
    pub fn crosses_up(&self, other: &BookSide) -> bool {
        let self_max_price = self.max_price();
        let other_min_price = other.min_price();
        match (self_max_price, other_min_price) {
            (Some(max_price), Some(min_price)) => max_price >= min_price,
            // can't cross if no prices (no orders)
            _ => false,
        }
    }
}

impl TryFrom<Vec<OrderSummary>> for BookSide {
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

impl From<BookSide> for Vec<OrderSummary> {
    fn from(_value: BookSide) -> Self {
        // TODO: Implement via OrderSummary::builder() (uses `bon` crate)
        todo!()
    }
}
