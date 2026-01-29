use crate::{Amount, ConditionId, ConvertVecTokenRawToTokensError, NegRisk, QuestionId, Rewards, TokenId, Tokens, TryFromNegRiskTripleError, from_chrono_date_time, into_chrono_date_time};
use alloy::primitives::Address;
use derive_more::{From, Into};
use polymarket_client_sdk::clob::types::response::{MarketResponse, Rewards as RewardsRaw, Token as TokenRaw};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Duration, OffsetDateTime};

#[derive(From, Into, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Market {
    pub question: String,
    pub description: String,
    pub market_slug: String,
    pub icon: String,
    pub image: String,
    /// Condition id provided by the API.
    pub condition_id: ConditionId,
    /// Question id provided by the API.
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
    pub fpmm: Option<Address>,
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

#[derive(Clone, Debug)]
pub struct FallibleMarket {
    pub question: String,
    pub description: String,
    pub market_slug: String,
    pub icon: String,
    pub image: String,
    /// Optional condition id provided by the API.
    pub condition_id: Option<ConditionId>,
    /// Optional question id provided by the API.
    pub question_id: Option<QuestionId>,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub enable_order_book: bool,
    pub accepting_orders: bool,
    pub accepting_order_timestamp: Result<Option<OffsetDateTime>, time::error::ComponentRange>,
    pub minimum_order_size: Amount,
    pub minimum_tick_size: Amount,
    pub end_date_iso: Result<Option<OffsetDateTime>, time::error::ComponentRange>,
    pub game_start_time: Result<Option<OffsetDateTime>, time::error::ComponentRange>,
    pub seconds_delay: Result<Duration, core::num::TryFromIntError>,
    pub fpmm: Option<Address>,
    pub maker_base_fee: Amount,
    pub taker_base_fee: Amount,
    pub rewards: Rewards,
    pub tokens: Result<Tokens, ConvertVecTokenRawToTokensError>,
    pub neg_risk: Result<Option<NegRisk>, TryFromNegRiskTripleError>,
    pub is_50_50_outcome: bool,
    pub notifications_enabled: bool,
    pub tags: Vec<String>,
}

/// NOTE: Some markets have an invalid `neg_risk_market_id` (e.g. "0x12309") because they were created by Polymarket just for testing
impl TryFrom<MarketResponse> for Market {
    type Error = ConvertMarketResponseToMarketError;

    fn try_from(market_response: MarketResponse) -> Result<Self, Self::Error> {
        use ConvertMarketResponseToMarketError::*;
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
        } = market_response;
        let rewards = rewards.into();
        let accepting_order_timestamp = accepting_order_timestamp
            .map(from_chrono_date_time)
            .transpose();
        let end_date_iso = end_date_iso.map(from_chrono_date_time).transpose();
        let game_start_time = game_start_time.map(from_chrono_date_time).transpose();
        let seconds_delay = i64::try_from(seconds_delay).map(Duration::seconds);
        let neg_risk = NegRisk::try_from_neg_risk_triple(neg_risk, neg_risk_market_id, neg_risk_request_id);
        let tokens = Tokens::try_from(tokens);
        match (condition_id, question_id, accepting_order_timestamp, end_date_iso, game_start_time, seconds_delay, neg_risk, tokens) {
            (Some(condition_id), Some(question_id), Ok(accepting_order_timestamp), Ok(end_date_iso), Ok(game_start_time), Ok(seconds_delay), Ok(neg_risk), Ok(tokens)) => Ok(Self {
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
            }),
            (condition_id, question_id, accepting_order_timestamp, end_date_iso, game_start_time, seconds_delay, neg_risk, tokens) => Err(ConversionFailed {
                fallible_market: FallibleMarket {
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
                },
            }),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConvertMarketResponseToMarketError {
    #[error("failed to convert market response")]
    ConversionFailed { fallible_market: FallibleMarket },
}

impl From<Market> for MarketResponse {
    fn from(market: Market) -> Self {
        let Market {
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
        } = market;
        let accepting_order_timestamp = accepting_order_timestamp.map(|timestamp| into_chrono_date_time(timestamp).expect("accepting_order_timestamp should convert because it originated from TryFrom"));
        let end_date_iso = end_date_iso.map(|timestamp| into_chrono_date_time(timestamp).expect("end_date_iso should convert because it originated from TryFrom"));
        let game_start_time = game_start_time.map(|timestamp| into_chrono_date_time(timestamp).expect("game_start_time should convert because it originated from TryFrom"));
        let seconds_delay = seconds_delay.whole_seconds();
        let seconds_delay = u64::try_from(seconds_delay).map_or(0, |value| value);
        let (neg_risk, neg_risk_market_id, neg_risk_request_id) = neg_risk
            .map(Into::into)
            .unwrap_or_else(|| (false, None, None));
        let rewards: RewardsRaw = rewards.into();
        let tokens: Vec<TokenRaw> = tokens.into();
        MarketResponse::builder()
            .enable_order_book(enable_order_book)
            .active(active)
            .closed(closed)
            .archived(archived)
            .accepting_orders(accepting_orders)
            .maybe_accepting_order_timestamp(accepting_order_timestamp)
            .minimum_order_size(minimum_order_size)
            .minimum_tick_size(minimum_tick_size)
            .maybe_condition_id(Some(condition_id))
            .maybe_question_id(Some(question_id))
            .question(question)
            .description(description)
            .market_slug(market_slug)
            .maybe_end_date_iso(end_date_iso)
            .maybe_game_start_time(game_start_time)
            .seconds_delay(seconds_delay)
            .maybe_fpmm(fpmm)
            .maker_base_fee(maker_base_fee)
            .taker_base_fee(taker_base_fee)
            .notifications_enabled(notifications_enabled)
            .neg_risk(neg_risk)
            .maybe_neg_risk_market_id(neg_risk_market_id)
            .maybe_neg_risk_request_id(neg_risk_request_id)
            .icon(icon)
            .image(image)
            .rewards(rewards)
            .is_50_50_outcome(is_50_50_outcome)
            .tokens(tokens)
            .tags(tags)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use errgonomic::{handle, handle_bool};

    #[test]
    fn must_round_trip_fixture() -> Result<(), MustRoundTripFixtureError> {
        use MustRoundTripFixtureError::*;
        let input = include_str!("../../fixtures/market.json");
        let market_response: MarketResponse = handle!(serde_json::de::from_str(input), DeserializeFailed);
        let market = handle!(Market::try_from(market_response.clone()), TryFromFailed);
        let expected_question = "Will Donald Trump win the 2024 US Presidential Election?".to_string();
        handle_bool!(
            market.question != expected_question,
            QuestionMismatch,
            actual: market.question.clone(),
            expected: expected_question
        );
        let market_response_round_trip = MarketResponse::from(market);
        handle_bool!(market_response_round_trip != market_response, RoundTripFailed, market_response, market_response_round_trip);
        Ok(())
    }

    #[allow(clippy::enum_variant_names)]
    #[derive(Error, Debug)]
    enum MustRoundTripFixtureError {
        #[error("failed to deserialize market fixture")]
        DeserializeFailed { source: serde_json::Error },
        #[error("failed to convert market response")]
        TryFromFailed { source: Box<ConvertMarketResponseToMarketError> },
        #[error("market question mismatch")]
        QuestionMismatch { actual: String, expected: String },
        #[error("round-tripped market response does not match original")]
        RoundTripFailed { market_response: Box<MarketResponse>, market_response_round_trip: Box<MarketResponse> },
    }
}
