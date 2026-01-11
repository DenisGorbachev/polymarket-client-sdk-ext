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

// TODO: Remove `clippy::infallible_try_from` after fixing the error handling
#[allow(clippy::infallible_try_from)]
impl TryFrom<Vec<OrderSummary>> for BookSide {
    type Error = TryFromVecOrderSummaryForBookSideError;

    fn try_from(_summaries: Vec<OrderSummary>) -> Result<Self, Self::Error> {
        // TODO: Return errors for OrderSummaries at a price level that already exists and has a different amount
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum TryFromVecOrderSummaryForBookSideError {}

// impl Serialize for BookSide {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let order_summaries: Vec<Level> = self.iter().map(Level::from).collect();
//         order_summaries.serialize(serializer)
//     }
// }
//
// impl<'de> Deserialize<'de> for BookSide {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let order_summaries: Vec<Level> = Vec::deserialize(deserializer)?;
//         let map = order_summaries.into_iter().map(Level::into).collect();
//         Ok(Self(map))
//     }
// }
