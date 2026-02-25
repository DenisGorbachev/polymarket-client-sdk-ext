use clap::ValueEnum;
use derive_more::From;
use polymarket_client_sdk::clob::types::OrderType as PolymarketClobOrderType;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, From, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum OrderType {
    #[serde(rename = "GTC")]
    Gtc,
    #[serde(rename = "FOK")]
    Fok,
    #[serde(rename = "GTD")]
    Gtd,
    #[serde(rename = "FAK")]
    Fak,
}

impl From<OrderType> for PolymarketClobOrderType {
    fn from(input: OrderType) -> Self {
        use OrderType::*;
        match input {
            Gtc => Self::GTC,
            Fok => Self::FOK,
            Gtd => Self::GTD,
            Fak => Self::FAK,
        }
    }
}
