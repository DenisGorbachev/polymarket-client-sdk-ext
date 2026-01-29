use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct QuestionIdIsNoneIffConditionIdIsNone;

impl Property<MarketResponse> for QuestionIdIsNoneIffConditionIdIsNone {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        value.question_id.is_none() == value.condition_id.is_none()
    }
}

register_property!(QuestionIdIsNoneIffConditionIdIsNone, MarketResponse, MARKET_RESPONSE_PROPERTIES);
