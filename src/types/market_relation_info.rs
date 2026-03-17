use crate::MarketExchange;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub struct MarketRelationInfo {
    pub exchange: MarketExchange,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    pub question: String,
}
