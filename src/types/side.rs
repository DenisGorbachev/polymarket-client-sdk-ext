use clap::ValueEnum;
use derive_more::From;
use polymarket_client_sdk::clob::types::Side as PolymarketClobSide;

#[derive(ValueEnum, From, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl From<Side> for PolymarketClobSide {
    fn from(side: Side) -> Self {
        match side {
            Side::Buy => PolymarketClobSide::Buy,
            Side::Sell => PolymarketClobSide::Sell,
        }
    }
}
