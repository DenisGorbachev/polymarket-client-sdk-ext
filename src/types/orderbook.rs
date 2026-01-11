use crate::{from_chrono_date_time, BidAskCrossError, BookSide, ConditionId, TimestampVisitor, TokenId, UintAsString};
use derive_more::{From, Into};
use polymarket_client_sdk::clob::types::response::OrderBookSummaryResponse;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stub_macro::stub;
use time::OffsetDateTime;

#[derive(From, Into, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Orderbook {
    /// `condition_id` uniquely identifies the market
    pub condition_id: ConditionId,
    #[serde(with = "UintAsString")]
    pub token_id: TokenId,
    pub bids: BookSide,
    pub asks: BookSide,
    pub min_order_size: Decimal,
    pub min_tick_size: Decimal,
    pub neg_risk: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(with = "TimestampVisitor")]
    pub updated_at: OffsetDateTime,
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

impl TryFrom<OrderBookSummaryResponse> for Orderbook {
    type Error = ();

    // TODO: Fix error handling
    fn try_from(response: OrderBookSummaryResponse) -> Result<Self, Self::Error> {
        let OrderBookSummaryResponse {
            market: _,
            asset_id: _,
            timestamp,
            hash,
            bids,
            asks,
            min_order_size,
            neg_risk,
            tick_size,
            ..
        } = response;
        let condition_id = stub!(ConditionId, "Convert from `market`");
        let token_id = stub!(TokenId, "Convert from `asset_id`");
        let updated_at = from_chrono_date_time(timestamp).unwrap();
        let bids = BookSide::try_from(bids).unwrap();
        let asks = BookSide::try_from(asks).unwrap();
        let min_tick_size = tick_size.into();
        Ok(Self {
            condition_id,
            token_id,
            bids,
            asks,
            min_order_size,
            min_tick_size,
            neg_risk,
            hash,
            updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[ignore]
    #[test]
    fn must_round_trip_serde() {
        let input = include_str!("../../fixtures/orderbook.json").trim();
        let orderbook_summary_response: OrderBookSummaryResponse = serde_json::de::from_str(input).unwrap();
        let orderbook = Orderbook::try_from(orderbook_summary_response).unwrap();
        assert_eq!(orderbook.hash, Some("6b57f28fe93242322f8836463d3266551166f90b".to_string()));
        let output = serde_json::ser::to_string_pretty(&orderbook).unwrap();
        assert_eq!(input, output);
    }
}
