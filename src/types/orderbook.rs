use crate::{BidAskCrossError, Book, ConditionId, TimestampVisitor, TokenId, UintAsString};
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(From, Into, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Orderbook {
    pub market: ConditionId,
    #[serde(with = "UintAsString")]
    pub asset_id: TokenId,
    #[serde(with = "TimestampVisitor")]
    pub timestamp: OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_order_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neg_risk: Option<bool>,
    pub hash: String,
    pub bids: Book,
    pub asks: Book,
}

impl Orderbook {
    pub fn validate(&self) -> Result<(), BidAskCrossError> {
        let max_bid_price_opt = self.bids.keys().max();
        let min_ask_price_opt = self.asks.keys().min();
        match (max_bid_price_opt, min_ask_price_opt) {
            (Some(max_bid_price), Some(min_ask_price)) if max_bid_price >= min_ask_price => Err(BidAskCrossError::new(*max_bid_price, *min_ask_price)),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn must_round_trip_serde() {
        let input = include_str!("../../fixtures/orderbook.json").trim();
        let orderbook: Orderbook = serde_json::de::from_str(input).unwrap();
        assert_eq!(orderbook.hash, "6b57f28fe93242322f8836463d3266551166f90b");
        let output = serde_json::ser::to_string_pretty(&orderbook).unwrap();
        assert_eq!(input, output);
    }
}
