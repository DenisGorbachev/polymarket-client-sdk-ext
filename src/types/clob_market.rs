use crate::types::market_response_precise::MarketResponsePrecise;
use crate::{Amount, ConditionId, EventId, NegRisk, QuestionId, Rewards, RkyvDecimal, RkyvOffsetDateTime, TokenId, Tokens, TryFromNegRiskTripleError, WinnerId};
use alloy::primitives::Address;
use derive_more::{From, Into};
use rkyv::with::Map;
use thiserror::Error;
use time::{Duration, OffsetDateTime};

#[derive(From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ClobMarket {
    pub question: String,
    pub description: String,
    pub slug: String,
    #[serde(with = "alloy::primitives::serde_hex")]
    pub condition_id: ConditionId,
    #[serde(with = "alloy::primitives::serde_hex")]
    pub question_id: QuestionId,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub enable_order_book: bool,
    pub accepting_orders: bool,
    #[rkyv(with = Map<RkyvOffsetDateTime>)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepting_order_timestamp: Option<OffsetDateTime>,
    #[rkyv(with = RkyvDecimal)]
    pub minimum_order_size: Amount,
    #[rkyv(with = RkyvDecimal)]
    pub minimum_tick_size: Amount,
    #[rkyv(with = Map<RkyvOffsetDateTime>)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fpmm: Option<Address>,
    #[rkyv(with = RkyvDecimal)]
    pub maker_base_fee: Amount,
    #[rkyv(with = RkyvDecimal)]
    pub taker_base_fee: Amount,
    pub left_token_id: TokenId,
    pub right_token_id: TokenId,
    pub winner_id: Option<WinnerId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neg_risk: Option<NegRisk>,
    pub is_50_50_outcome: bool,
}

impl ClobMarket {
    pub fn maybe_try_from_market_response_precise(market_response: MarketResponsePrecise) -> Option<Result<Self, <Self as TryFrom<MarketResponsePrecise>>::Error>> {
        if market_response.is_skipped() { None } else { Some(Self::try_from(market_response)) }
    }
}

impl TryFrom<MarketResponsePrecise> for ClobMarket {
    type Error = ClobMarketFallible;

    fn try_from(market_response: MarketResponsePrecise) -> Result<Self, Self::Error> {
        let MarketResponsePrecise {
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
            neg_risk: neg_risk_flag,
            neg_risk_market_id,
            neg_risk_request_id,
            is_50_50_outcome,
            notifications_enabled,
            tags,
        } = market_response;
        let (left_token_id, right_token_id) = tokens.token_ids_tuple();
        let winner_id = tokens.winner_id();
        let neg_risk_result = NegRisk::try_from_neg_risk_triple(neg_risk_flag, neg_risk_market_id, neg_risk_request_id);
        match (condition_id, question_id, neg_risk_result) {
            (Some(condition_id), Some(question_id), Ok(neg_risk)) => Ok(Self {
                question,
                description,
                slug: market_slug,
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
                end_date: end_date_iso,
                fpmm,
                maker_base_fee,
                taker_base_fee,
                left_token_id,
                right_token_id,
                winner_id,
                neg_risk,
                is_50_50_outcome,
            }),
            (condition_id, question_id, neg_risk) => Err(ClobMarketFallible {
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
                neg_risk_flag,
                winner_id,
                neg_risk,
                neg_risk_market_id,
                neg_risk_request_id,
                is_50_50_outcome,
                notifications_enabled,
                tags,
            }),
        }
    }
}

#[derive(Error, Debug)]
#[error("failed to convert market response to market")]
pub struct ClobMarketFallible {
    pub question: String,
    pub description: String,
    pub market_slug: String,
    pub icon: String,
    pub image: String,
    pub condition_id: Option<ConditionId>,
    pub question_id: Option<QuestionId>,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub enable_order_book: bool,
    pub accepting_orders: bool,
    pub accepting_order_timestamp: Option<OffsetDateTime>,
    pub minimum_order_size: Amount,
    pub minimum_tick_size: Amount,
    pub end_date_iso: Option<OffsetDateTime>,
    pub game_start_time: Option<OffsetDateTime>,
    pub seconds_delay: Duration,
    pub fpmm: Option<Address>,
    pub maker_base_fee: Amount,
    pub taker_base_fee: Amount,
    pub rewards: Rewards,
    pub tokens: Tokens,
    pub winner_id: Option<WinnerId>,
    pub neg_risk_flag: bool,
    pub neg_risk: Result<Option<NegRisk>, TryFromNegRiskTripleError>,
    pub neg_risk_market_id: Option<QuestionId>,
    pub neg_risk_request_id: Option<EventId>,
    pub is_50_50_outcome: bool,
    pub notifications_enabled: bool,
    pub tags: Vec<String>,
}
