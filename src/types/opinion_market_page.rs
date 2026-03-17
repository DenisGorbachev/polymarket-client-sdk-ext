use crate::OpinionMarket;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub struct OpinionMarketPage {
    pub data: Vec<OpinionMarket>,
}
