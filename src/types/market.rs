use crate::{Amount, ConditionId, ConvertVecTokenRawToTokensError, NegRisk, QuestionId, Rewards, TokenId, Tokens, TryFromNegRiskTripleError, from_chrono_date_time, into_chrono_date_time};
use alloy::primitives::Address;
use derive_more::{From, Into};
use errgonomic::handle;
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
    /// This field can be equal to [`Address::ZERO`]
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
impl TryFrom<MarketResponse> for Market {
    type Error = ConvertMarketResponseToMarketError;

    fn try_from(market: MarketResponse) -> Result<Self, Self::Error> {
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
        } = market;
        let condition_id = handle!(condition_id.parse::<ConditionId>(), ConditionIdParseFailed, condition_id);
        let question_id = handle!(question_id.parse::<QuestionId>(), QuestionIdParseFailed, question_id);
        let rewards = rewards.into();
        let accepting_order_timestamp = handle!(
            accepting_order_timestamp
                .map(from_chrono_date_time)
                .transpose(),
            AcceptingOrderTimestampFromChronoDateTimeFailed
        );
        let end_date_iso = handle!(end_date_iso.map(from_chrono_date_time).transpose(), EndDateIsoFromChronoDateTimeFailed);
        let game_start_time = handle!(game_start_time.map(from_chrono_date_time).transpose(), GameStartTimeFromChronoDateTimeFailed);
        let seconds_delay = handle!(i64::try_from(seconds_delay), SecondsDelayTryFromFailed, seconds_delay);
        let seconds_delay = Duration::seconds(seconds_delay);
        let fpmm = if fpmm.is_empty() {
            Address::ZERO
        } else {
            handle!(fpmm.parse::<Address>(), FpmmParseFailed, fpmm)
        };
        let neg_risk = handle!(NegRisk::try_from_neg_risk_triple(neg_risk, neg_risk_market_id, neg_risk_request_id), NegRiskTryFromTripleFailed);
        let tokens = handle!(Tokens::try_from(tokens), TokensTryFromFailed);
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

#[derive(Error, Debug)]
pub enum ConvertMarketResponseToMarketError {
    #[error("failed to parse condition_id '{condition_id}'")]
    ConditionIdParseFailed { source: alloy::hex::FromHexError, condition_id: String },
    #[error("failed to parse question_id '{question_id}'")]
    QuestionIdParseFailed { source: alloy::hex::FromHexError, question_id: String },
    #[error("failed to convert accepting_order_timestamp")]
    AcceptingOrderTimestampFromChronoDateTimeFailed { source: time::error::ComponentRange },
    #[error("failed to convert end_date_iso")]
    EndDateIsoFromChronoDateTimeFailed { source: time::error::ComponentRange },
    #[error("failed to convert game_start_time")]
    GameStartTimeFromChronoDateTimeFailed { source: time::error::ComponentRange },
    #[error("failed to convert seconds_delay '{seconds_delay}'")]
    SecondsDelayTryFromFailed { source: core::num::TryFromIntError, seconds_delay: u64 },
    #[error("failed to parse fpmm '{fpmm}'")]
    FpmmParseFailed { source: alloy::hex::FromHexError, fpmm: String },
    #[error("failed to convert neg_risk fields")]
    NegRiskTryFromTripleFailed { source: TryFromNegRiskTripleError },
    #[error("failed to convert tokens")]
    TokensTryFromFailed { source: ConvertVecTokenRawToTokensError },
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
        let condition_id = condition_id.to_string();
        let question_id = question_id.to_string();
        let accepting_order_timestamp = accepting_order_timestamp.map(|timestamp| into_chrono_date_time(timestamp).expect("accepting_order_timestamp should convert because it originated from TryFrom"));
        let end_date_iso = end_date_iso.map(|timestamp| into_chrono_date_time(timestamp).expect("end_date_iso should convert because it originated from TryFrom"));
        let game_start_time = game_start_time.map(|timestamp| into_chrono_date_time(timestamp).expect("game_start_time should convert because it originated from TryFrom"));
        let seconds_delay = seconds_delay.whole_seconds();
        let seconds_delay = u64::try_from(seconds_delay).map_or(0, |value| value);
        let fpmm = if fpmm == Address::ZERO { String::new() } else { fpmm.to_string() };
        let (neg_risk, neg_risk_market_id, neg_risk_request_id) = neg_risk
            .map(Into::into)
            .unwrap_or_else(|| (false, String::new(), String::new()));
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
            .condition_id(condition_id)
            .question_id(question_id)
            .question(question)
            .description(description)
            .market_slug(market_slug)
            .maybe_end_date_iso(end_date_iso)
            .maybe_game_start_time(game_start_time)
            .seconds_delay(seconds_delay)
            .fpmm(fpmm)
            .maker_base_fee(maker_base_fee)
            .taker_base_fee(taker_base_fee)
            .notifications_enabled(notifications_enabled)
            .neg_risk(neg_risk)
            .neg_risk_market_id(neg_risk_market_id)
            .neg_risk_request_id(neg_risk_request_id)
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
    use crate::assert_round_trip_own;
    use futures::{Stream, stream};
    use std::process::ExitCode;

    #[test]
    fn must_round_trip_fixture() {
        let input = include_str!("../../fixtures/market.json");
        let market_response: MarketResponse = serde_json::de::from_str(input).unwrap();
        let market = Market::try_from(market_response.clone()).unwrap();
        assert_eq!(market.question, "Will Donald Trump win the 2024 US Presidential Election?");
        let market_response_round_trip = MarketResponse::from(market);
        assert_eq!(market_response_round_trip, market_response);
    }

    #[ignore]
    #[tokio::test]
    async fn must_round_trip_data() -> ExitCode {
        let inputs = get_market_response_stream();
        assert_round_trip_own::<MarketResponse, Market, <Market as TryFrom<MarketResponse>>::Error>(inputs).await
    }

    fn get_market_response_stream() -> impl Stream<Item = MarketResponse> {
        // TODO
        stream::empty()
    }
}
