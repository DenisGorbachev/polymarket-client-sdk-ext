use crate::{EventId, QuestionId};
use derive_more::{From, Into};
use derive_new::new;
use errgonomic::handle_bool;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(new, From, Into, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct NegRisk {
    /// Original name: `neg_risk_market_id`
    /// Examples: `0xe3b1bc389210504ebcb9cffe4b0ed06ccac50561e0f24abb6379984cec030f00`
    pub question_id: QuestionId,
    /// The id of an event that groups the markets (note: this is actually NOT a request id, but an event id)
    /// Original name: `neg_risk_request_id`
    /// Examples: `0xc2d6714f691eacd6ec494c7d6e5eaaf7dfba8907dcaf55b2dd93e7b479da1605`
    pub event_id: EventId,
}

impl NegRisk {
    // TODO: Fix error handling
    pub fn try_from_neg_risk_triple(neg_risk: bool, neg_risk_market_id: String, neg_risk_request_id: String) -> Result<Option<Self>, TryFromNegRiskTripleError> {
        use TryFromNegRiskTripleError::*;
        if neg_risk {
            let question_id = neg_risk_market_id.parse().unwrap();
            let event_id = neg_risk_request_id.parse().unwrap();
            Ok(Some(Self {
                question_id,
                event_id,
            }))
        } else {
            handle_bool!(!neg_risk_market_id.is_empty(), NegRiskMarketIdIsNotEmpty, neg_risk_market_id);
            handle_bool!(!neg_risk_request_id.is_empty(), NegRiskRequestIdIsNotEmpty, neg_risk_request_id);
            Ok(None)
        }
    }
}

impl From<NegRisk> for (bool, String, String) {
    fn from(value: NegRisk) -> Self {
        (true, value.question_id.to_string(), value.event_id.to_string())
    }
}

#[derive(Error, Debug)]
pub enum TryFromNegRiskTripleError {
    #[error("expected neg_risk_market_id to be empty, but it was not: '{neg_risk_market_id}'")]
    NegRiskMarketIdIsNotEmpty { neg_risk_market_id: String },
    #[error("expected neg_risk_request_id to be empty, but it was not: '{neg_risk_request_id}'")]
    NegRiskRequestIdIsNotEmpty { neg_risk_request_id: String },
}
