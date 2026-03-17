use crate::MarketRelationInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub struct MarketRelation {
    pub a: MarketRelationInfo,
    pub b: MarketRelationInfo,
    pub relation: String,
}
