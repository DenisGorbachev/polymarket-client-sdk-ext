use crate::{ConditionId, QuestionId};
use derive_more::{From, Into};
use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(new, From, Into, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct NegRisk {
    pub condition_id: ConditionId,
    pub question_id: QuestionId,
}

impl NegRisk {}
