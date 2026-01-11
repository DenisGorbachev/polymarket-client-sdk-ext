use crate::{BidAskCrossError, BookSide, ConditionId, TimestampVisitor, TokenId, UintAsString, from_chrono_date_time};
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

impl From<Orderbook> for OrderBookSummaryResponse {
    fn from(orderbook: Orderbook) -> Self {
        let Orderbook {
            condition_id: _,
            token_id: _,
            bids: _,
            asks: _,
            min_order_size: _,
            min_tick_size: _,
            neg_risk: _,
            hash: _,
            updated_at: _,
        } = orderbook;
        todo!()
        // cannot create non-exhaustive struct using struct expression [E0639]
        // Self {
        //     market: condition_id.to_string(),
        //     asset_id: token_id.to_string(),
        //     timestamp: into_chrono_date_time(updated_at),
        //     hash,
        //     bids: bids.into(),
        //     asks: asks.into(),
        //     min_order_size,
        //     neg_risk,
        //     tick_size: TickSize::try_from(min_tick_size).expect("min_tick_size should convert to tick_size without an error because it has been converted from tick_size in the TryFrom impl"),
        // }
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
        let orderbook = Orderbook::try_from(orderbook_summary_response.clone()).unwrap();
        let output = serde_json::ser::to_string_pretty(&orderbook).unwrap();
        assert_eq!(input, output);
    }
}

mod unused {
    #![allow(dead_code)]

    use errgonomic::{handle, handle_bool};
    use futures::Stream;
    use futures::StreamExt;
    use std::error::Error;
    use thiserror::Error;

    async fn try_round_trip<In, Out, Err>(inputs: impl Stream<Item = In>) -> impl Stream<Item = Result<Out, RoundTripError<In, Err>>>
    where
        In: for<'a> From<&'a Out> + PartialEq,
        Out: for<'a> TryFrom<&'a In, Error = Err>,
        Err: Error,
    {
        inputs.map(|input| {
            use RoundTripError::*;
            let output = handle!(Out::try_from(&input), TryFromFailed, input);
            let input_round_trip = In::from(&output);
            handle_bool!(input != input_round_trip, RoundTripFailed, input, input_round_trip);
            Ok(output)
        })
    }

    #[derive(Error, Debug)]
    pub enum RoundTripError<In, Err>
    where
        Err: Error,
    {
        TryFromFailed { source: Err, input: In },
        RoundTripFailed { input: In, input_round_trip: In },
    }
}
