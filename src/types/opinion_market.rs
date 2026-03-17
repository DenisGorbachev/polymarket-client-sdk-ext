use crate::{MarketExchange, MarketRelationInfo};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpinionMarket {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub child_markets: Vec<Self>,
}

impl OpinionMarket {
    pub fn into_market_relation_infos(self) -> Vec<MarketRelationInfo> {
        use MarketExchange::*;
        let Self {
            id,
            slug,
            title,
            child_markets,
        } = self;
        let current_info_opt = title.map(|question| MarketRelationInfo {
            exchange: Opinion,
            id,
            slug,
            question,
        });
        let child_infos = child_markets
            .into_iter()
            .flat_map(Self::into_market_relation_infos);
        current_info_opt.into_iter().chain(child_infos).collect()
    }
}
