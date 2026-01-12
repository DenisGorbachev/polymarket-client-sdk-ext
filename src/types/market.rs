use crate::{Amount, ConditionId, NegRisk, QuestionId, Rewards, TokenId, Tokens, from_chrono_date_time};
use alloy::primitives::Address;
use derive_more::{From, Into};
use polymarket_client_sdk::clob::types::response::MarketResponse;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

#[derive(From, Into, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Market {
    pub question: String,
    pub description: String,
    pub market_slug: String,
    pub icon: String,
    pub image: String,
    pub condition_id: ConditionId,
    pub question_id: QuestionId,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub enable_order_book: bool,
    pub accepting_orders: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepting_order_timestamp: Option<OffsetDateTime>,
    pub minimum_order_size: Amount,
    pub minimum_tick_size: Amount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date_iso: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_start_time: Option<OffsetDateTime>,
    pub seconds_delay: Duration,
    pub fpmm: Address,
    pub maker_base_fee: Amount,
    pub taker_base_fee: Amount,
    pub rewards: Rewards,
    pub tokens: Tokens,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neg_risk: Option<NegRisk>,
    pub is_50_50_outcome: bool,
    pub notifications_enabled: bool,
    pub tags: Vec<String>,
}

impl Market {
    pub fn is_tradeable(&self) -> bool {
        self.active && !self.closed && !self.archived && self.accepting_orders && self.enable_order_book
    }

    pub fn token_ids_tuple(&self) -> (TokenId, TokenId) {
        self.tokens.token_ids_tuple()
    }

    pub fn token_ids_array(&self) -> [TokenId; 2] {
        self.tokens.token_ids_array()
    }
}

/// NOTE: Some markets have an invalid `neg_risk_market_id` (e.g. "0x12309") because they were created by Polymarket just for testing
// TODO: Fix error handling
impl TryFrom<MarketResponse> for Market {
    type Error = ();

    fn try_from(market: MarketResponse) -> Result<Self, Self::Error> {
        let MarketResponse {
            enable_order_book,
            active,
            closed,
            archived,
            accepting_orders,
            accepting_order_timestamp,
            minimum_order_size,
            minimum_tick_size,
            condition_id,
            question_id,
            question,
            description,
            market_slug,
            end_date_iso,
            game_start_time,
            seconds_delay,
            fpmm,
            maker_base_fee,
            taker_base_fee,
            notifications_enabled,
            neg_risk,
            neg_risk_market_id,
            neg_risk_request_id,
            icon,
            image,
            rewards,
            is_50_50_outcome,
            tokens,
            tags,
            ..
        } = market;
        let condition_id = condition_id.parse::<ConditionId>().unwrap();
        let question_id = question_id.parse::<QuestionId>().unwrap();
        let rewards = rewards.into();
        let neg_risk = NegRisk::try_from_neg_risk_triple(neg_risk, neg_risk_market_id, neg_risk_request_id).unwrap();
        let accepting_order_timestamp = accepting_order_timestamp
            .map(from_chrono_date_time)
            .transpose()
            .unwrap();
        let end_date_iso = end_date_iso.map(from_chrono_date_time).transpose().unwrap();
        let game_start_time = game_start_time
            .map(from_chrono_date_time)
            .transpose()
            .unwrap();
        let seconds_delay = Duration::seconds(seconds_delay as i64);
        let fpmm = fpmm.parse::<Address>().unwrap();
        let tokens = Tokens::try_from(tokens).unwrap();
        Ok(Self {
            question,
            description,
            market_slug,
            icon,
            image,
            condition_id,
            question_id,
            active,
            closed,
            archived,
            enable_order_book,
            accepting_orders,
            accepting_order_timestamp,
            minimum_order_size,
            minimum_tick_size,
            end_date_iso,
            game_start_time,
            seconds_delay,
            fpmm,
            maker_base_fee,
            taker_base_fee,
            rewards,
            tokens,
            neg_risk,
            is_50_50_outcome,
            notifications_enabled,
            tags,
        })
    }
}

impl From<Market> for MarketResponse {
    fn from(_value: Market) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn must_round_trip() {
        let input = include_str!("../../fixtures/market.json");
        let market_response: MarketResponse = serde_json::de::from_str(input).unwrap();
        let market = Market::try_from(market_response.clone()).unwrap();
        assert_eq!(market.question, "Will Donald Trump win the 2024 US Presidential Election?");
        let market_response_round_trip = MarketResponse::from(market);
        assert_eq!(market_response_round_trip, market_response);
    }
}
