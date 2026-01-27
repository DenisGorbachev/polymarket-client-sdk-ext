use crate::Holds;
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct QuestionIdIsNoneIffConditionIdIsNone;

impl Holds<MarketResponse> for QuestionIdIsNoneIffConditionIdIsNone {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        value.question_id.is_none() == value.condition_id.is_none()
    }
}
