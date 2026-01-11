use crate::{Amount, ConditionId, MarketRaw, QuestionId, Rewards, TokenId, Tokens};
use alloy::primitives::Address;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

/// NOTE: [`MarketRaw`] contains more fields (e.g. `neg_risk*`, `accepting_order_timestamp`)
#[derive(From, Into, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
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

    pub fn token_ids_vec(&self) -> Vec<TokenId> {
        self.tokens.token_ids_vec()
    }
}

impl TryFrom<MarketRaw> for Market {
    type Error = ();

    fn try_from(value: MarketRaw) -> Result<Self, Self::Error> {
        let MarketRaw {
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
            neg_risk: _,
            neg_risk_market_id: _,
            neg_risk_request_id: _,
            rewards,
            tokens,
            is_50_50_outcome,
            notifications_enabled,
            tags,
        } = value;
        let condition_id = condition_id.ok_or(())?;
        let question_id = question_id.ok_or(())?;
        let rewards = rewards.try_into()?;
        let tags = tags.unwrap_or_default();
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
            is_50_50_outcome,
            notifications_enabled,
            tags,
        })
    }
}
