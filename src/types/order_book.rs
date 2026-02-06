use crate::RkyvOffsetDateTime;
use crate::{BidAskCrossError, BookSideMap, ConditionId, ConvertVecOrderSummaryToBookSideError, RkyvDecimal, TimestampVisitor, TokenId, UintAsString, from_chrono_date_time, into_chrono_date_time};
use derive_more::{From, Into};
use errgonomic::handle;
use polymarket_client_sdk::clob::types::TickSize;
use polymarket_client_sdk::clob::types::response::OrderBookSummaryResponse;
use rkyv::with::Map;
use rust_decimal::Decimal;
use thiserror::Error;
use time::OffsetDateTime;

#[derive(From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct OrderBookSummaryResponsePrecise {
    /// `condition_id` uniquely identifies the market
    #[serde(with = "alloy::primitives::serde_hex")]
    pub condition_id: ConditionId,
    #[serde(with = "UintAsString")]
    pub token_id: TokenId,
    #[serde(with = "TimestampVisitor")]
    #[rkyv(with = RkyvOffsetDateTime)]
    pub updated_at: OffsetDateTime,
    pub hash: Option<String>,
    #[serde(with = "rust_decimal::serde::str_option")]
    #[rkyv(with = Map<RkyvDecimal>)]
    pub last_trade_price: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str")]
    #[rkyv(with = RkyvDecimal)]
    pub min_order_size: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    #[rkyv(with = RkyvDecimal)]
    pub min_tick_size: Decimal,
    pub neg_risk: bool,
    pub bids: BookSideMap,
    pub asks: BookSideMap,
}

impl OrderBookSummaryResponsePrecise {
    pub fn validate(&self) -> Result<(), BidAskCrossError> {
        let max_bid_price_opt = self.bids.keys().max();
        let min_ask_price_opt = self.asks.keys().min();
        match (max_bid_price_opt, min_ask_price_opt) {
            (Some(max_bid_price), Some(min_ask_price)) if max_bid_price >= min_ask_price => Err(BidAskCrossError::new(*max_bid_price, *min_ask_price)),
            _ => Ok(()),
        }
    }
}

impl TryFrom<OrderBookSummaryResponse> for OrderBookSummaryResponsePrecise {
    type Error = ConvertOrderBookSummaryResponseToOrderbookError;

    fn try_from(response: OrderBookSummaryResponse) -> Result<Self, Self::Error> {
        use ConvertOrderBookSummaryResponseToOrderbookError::*;
        let OrderBookSummaryResponse {
            market,
            asset_id,
            timestamp,
            hash,
            bids,
            asks,
            min_order_size,
            neg_risk,
            tick_size,
            last_trade_price,
            ..
        } = response;
        let condition_id = market;
        let token_id = asset_id;
        let updated_at = handle!(from_chrono_date_time(timestamp), FromChronoDateTimeFailed, timestamp);
        let bids = handle!(BookSideMap::try_from(bids), BidsTryFromFailed);
        let asks = handle!(BookSideMap::try_from(asks), AsksTryFromFailed);
        let min_tick_size = tick_size.into();
        Ok(Self {
            condition_id,
            token_id,
            updated_at,
            min_order_size,
            min_tick_size,
            neg_risk,
            hash,
            last_trade_price,
            bids,
            asks,
        })
    }
}

#[derive(Error, Debug)]
pub enum ConvertOrderBookSummaryResponseToOrderbookError {
    #[error("failed to convert timestamp '{timestamp}'")]
    FromChronoDateTimeFailed { source: time::error::ComponentRange, timestamp: chrono::DateTime<chrono::Utc> },
    #[error("failed to convert bids")]
    BidsTryFromFailed { source: ConvertVecOrderSummaryToBookSideError },
    #[error("failed to convert asks")]
    AsksTryFromFailed { source: ConvertVecOrderSummaryToBookSideError },
}

impl From<OrderBookSummaryResponsePrecise> for OrderBookSummaryResponse {
    fn from(orderbook: OrderBookSummaryResponsePrecise) -> Self {
        let OrderBookSummaryResponsePrecise {
            condition_id,
            token_id,
            bids,
            asks,
            min_order_size,
            min_tick_size,
            neg_risk,
            hash,
            updated_at,
            last_trade_price,
        } = orderbook;
        let market = condition_id;
        let asset_id = token_id;
        let timestamp = into_chrono_date_time(updated_at).expect("timestamp should convert an error because it has been converted timestamp in the TryFrom impl");
        let tick_size = TickSize::try_from(min_tick_size).expect("min_tick_size should convert to tick_size without an error because it has been converted from tick_size in the TryFrom impl");
        OrderBookSummaryResponse::builder()
            .market(market)
            .asset_id(asset_id)
            .timestamp(timestamp)
            .maybe_hash(hash)
            .maybe_last_trade_price(last_trade_price)
            .bids(bids.into())
            .asks(asks.into())
            .min_order_size(min_order_size)
            .neg_risk(neg_risk)
            .tick_size(tick_size)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use errgonomic::{handle, handle_bool};

    #[test]
    fn must_round_trip() -> Result<(), MustRoundTripFixtureError> {
        use MustRoundTripFixtureError::*;
        let input = include_str!("../../fixtures/orderbook.json").trim();
        let orderbook_summary_response = handle!(serde_json::de::from_str::<OrderBookSummaryResponse>(input), DeserializeFailed);
        let orderbook = handle!(OrderBookSummaryResponsePrecise::try_from(orderbook_summary_response.clone()), TryFromFailed, orderbook_summary_response);
        let orderbook_summary_response_round_trip = OrderBookSummaryResponse::from(orderbook);
        handle_bool!(orderbook_summary_response_round_trip != orderbook_summary_response, RoundTripFailed, orderbook_summary_response, orderbook_summary_response_round_trip);
        Ok(())
    }

    #[allow(clippy::enum_variant_names)]
    #[derive(Error, Debug)]
    enum MustRoundTripFixtureError {
        #[error("failed to deserialize orderbook fixture")]
        DeserializeFailed { source: serde_json::Error },
        #[error("failed to convert orderbook response")]
        TryFromFailed { source: ConvertOrderBookSummaryResponseToOrderbookError, orderbook_summary_response: Box<OrderBookSummaryResponse> },
        #[error("round-tripped orderbook response does not match original")]
        RoundTripFailed { orderbook_summary_response: Box<OrderBookSummaryResponse>, orderbook_summary_response_round_trip: Box<OrderBookSummaryResponse> },
    }
}
