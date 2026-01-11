use crate::{EventId, QuestionId};
use derive_more::{From, Into};
use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(new, From, Into, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct NegRisk {
    /// The id of an event that groups the markets
    /// Original name: `neg_risk_market_id` (note: this is actually NOT a market id, but an event id)
    pub event_id: EventId,
    /// Originally called `neg_risk_request_id`
    pub question_id: QuestionId,
}

impl NegRisk {}
