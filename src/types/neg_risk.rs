use crate::{EventId, QuestionId};
use alloy::primitives::B256;
use derive_more::{From, Into};
use derive_new::new;
use errgonomic::{handle_opt, handle_opt_take};
use thiserror::Error;

#[derive(new, From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
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
    pub fn try_from_neg_risk_triple(neg_risk: bool, mut neg_risk_market_id: Option<B256>, mut neg_risk_request_id: Option<B256>) -> Result<Option<Self>, TryFromNegRiskTripleError> {
        use TryFromNegRiskTripleError::*;
        if neg_risk {
            let question_id = handle_opt!(neg_risk_market_id, NegRiskMarketIdIsNone);
            let event_id = handle_opt!(neg_risk_request_id, NegRiskRequestIdIsNone);
            Ok(Some(Self {
                question_id,
                event_id,
            }))
        } else {
            handle_opt_take!(neg_risk_market_id, NegRiskMarketIdIsNotNone, neg_risk_market_id);
            handle_opt_take!(neg_risk_request_id, NegRiskRequestIdIsNotNone, neg_risk_request_id);
            Ok(None)
        }
    }
}

impl From<NegRisk> for (bool, Option<B256>, Option<B256>) {
    fn from(value: NegRisk) -> Self {
        (true, Some(value.question_id), Some(value.event_id))
    }
}

#[derive(Error, Clone, Debug)]
pub enum TryFromNegRiskTripleError {
    #[error("expected neg_risk_market_id to be Some(value), but it was None")]
    NegRiskMarketIdIsNone,
    #[error("expected neg_risk_request_id to be Some(value), but it was None")]
    NegRiskRequestIdIsNone,
    #[error("expected neg_risk_market_id to be None, but it was Some('{}')", display_b256(neg_risk_market_id))]
    NegRiskMarketIdIsNotNone { neg_risk_market_id: B256 },
    #[error("expected neg_risk_request_id to be None, but it was Some('{}')", display_b256(neg_risk_request_id))]
    NegRiskRequestIdIsNotNone { neg_risk_request_id: B256 },
}

// TODO: Replace calls to this fn with calls to a corresponding fn in `alloy` crate (find the right fn in `alloy`)
pub fn display_b256(_input: &B256) -> String {
    todo!()
}
