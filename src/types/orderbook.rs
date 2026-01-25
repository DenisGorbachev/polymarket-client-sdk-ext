use crate::{BidAskCrossError, BookSide, ConditionId, ConvertVecOrderSummaryToBookSideError, TimestampVisitor, TokenId, UintAsString, from_chrono_date_time, into_chrono_date_time};
use derive_more::{From, Into};
use errgonomic::handle;
use polymarket_client_sdk::clob::types::TickSize;
use polymarket_client_sdk::clob::types::response::OrderBookSummaryResponse;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(From, Into, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Orderbook {
    /// `condition_id` uniquely identifies the market
    #[serde(with = "alloy::primitives::serde_hex")]
    pub condition_id: ConditionId,
    #[serde(with = "UintAsString")]
    pub token_id: TokenId,
    #[serde(with = "TimestampVisitor")]
    pub updated_at: OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    pub min_order_size: Decimal,
    pub min_tick_size: Decimal,
    pub neg_risk: bool,
    pub bids: BookSide,
    pub asks: BookSide,
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
            ..
        } = response;
        let condition_id = market;
        let token_id = asset_id;
        let updated_at = handle!(from_chrono_date_time(timestamp), FromChronoDateTimeFailed, timestamp);
        let bids = handle!(BookSide::try_from(bids), BidsTryFromFailed);
        let asks = handle!(BookSide::try_from(asks), AsksTryFromFailed);
        let min_tick_size = tick_size.into();
        Ok(Self {
            condition_id,
            token_id,
            updated_at,
            min_order_size,
            min_tick_size,
            neg_risk,
            hash,
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

impl From<Orderbook> for OrderBookSummaryResponse {
    fn from(orderbook: Orderbook) -> Self {
        let Orderbook {
            condition_id,
            token_id,
            bids,
            asks,
            min_order_size,
            min_tick_size,
            neg_risk,
            hash,
            updated_at,
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
    use crate::{ReadJsonlCacheStreamError, orderbook_summary_response_cache_path, read_jsonl_cache_stream};
    use errgonomic::{exit_stream_of_results_print_first, handle, handle_bool};
    use futures::{Stream, StreamExt};
    use std::process::ExitCode;

    #[test]
    fn must_round_trip() -> Result<(), MustRoundTripFixtureError> {
        use MustRoundTripFixtureError::*;
        let input = include_str!("../../fixtures/orderbook.json").trim();
        let orderbook_summary_response = handle!(serde_json::de::from_str::<OrderBookSummaryResponse>(input), DeserializeFailed);
        let orderbook = handle!(Orderbook::try_from(orderbook_summary_response.clone()), TryFromFailed, orderbook_summary_response);
        let orderbook_summary_response_round_trip = OrderBookSummaryResponse::from(orderbook);
        handle_bool!(orderbook_summary_response_round_trip != orderbook_summary_response, RoundTripFailed, orderbook_summary_response, orderbook_summary_response_round_trip);
        Ok(())
    }

    #[tokio::test]
    async fn must_round_trip_cache() -> ExitCode {
        let inputs = get_orderbook_summary_response_stream().await;
        let results = inputs.map(|result| {
            use MustRoundTripCacheError::*;
            let input = handle!(result, ReadJsonlCacheStreamFailed);
            let output = handle!(Orderbook::try_from(input.clone()), TryFromFailed, input);
            let input_round_trip = OrderBookSummaryResponse::from(output);
            handle_bool!(input != input_round_trip, RoundTripFailed, input, input_round_trip);
            Ok(())
        });
        exit_stream_of_results_print_first(results).await
    }

    async fn get_orderbook_summary_response_stream() -> impl Stream<Item = Result<OrderBookSummaryResponse, ReadJsonlCacheStreamError>> {
        let cache_path = orderbook_summary_response_cache_path();
        match read_jsonl_cache_stream::<OrderBookSummaryResponse>(cache_path).await {
            Ok(stream) => stream.boxed(),
            Err(error) => futures::stream::once(async move { Err(error) }).boxed(),
        }
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

    #[allow(clippy::enum_variant_names)]
    #[derive(Error, Debug)]
    enum MustRoundTripCacheError {
        #[error("failed to read orderbook cache stream")]
        ReadJsonlCacheStreamFailed { source: ReadJsonlCacheStreamError },
        #[error("failed to convert orderbook response")]
        TryFromFailed { source: ConvertOrderBookSummaryResponseToOrderbookError, input: Box<OrderBookSummaryResponse> },
        #[error("round-tripped orderbook response does not match original")]
        RoundTripFailed { input: Box<OrderBookSummaryResponse>, input_round_trip: Box<OrderBookSummaryResponse> },
    }
}
