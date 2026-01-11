use crate::{BidAskCrossError, BookSide, ConditionId, ConvertVecOrderSummaryToBookSideError, TimestampVisitor, TokenId, UintAsString, from_chrono_date_time};
use derive_more::{From, Into};
use errgonomic::handle;
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
        let condition_id = handle!(market.parse::<ConditionId>(), MarketParseFailed, market);
        let token_id = handle!(asset_id.parse::<TokenId>(), AssetIdParseFailed, asset_id);
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
    #[error("failed to parse condition id from market '{market}'")]
    MarketParseFailed { source: alloy::hex::FromHexError, market: String },
    #[error("failed to parse token id from asset id '{asset_id}'")]
    AssetIdParseFailed { source: alloy::primitives::ruint::ParseError, asset_id: String },
    #[error("failed to convert timestamp '{timestamp}'")]
    FromChronoDateTimeFailed { source: time::error::ComponentRange, timestamp: chrono::DateTime<chrono::Utc> },
    #[error("failed to convert bids")]
    BidsTryFromFailed { source: ConvertVecOrderSummaryToBookSideError },
    #[error("failed to convert asks")]
    AsksTryFromFailed { source: ConvertVecOrderSummaryToBookSideError },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn must_round_trip_serde() {
        let input = include_str!("../../fixtures/orderbook.json").trim();
        let orderbook_summary_response: OrderBookSummaryResponse = serde_json::de::from_str(input).unwrap();
        let _orderbook = Orderbook::try_from(orderbook_summary_response).unwrap();
    }
}
