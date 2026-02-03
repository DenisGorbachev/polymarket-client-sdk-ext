use crate::GammaMarket;
use derive_more::{From, Into};
use polymarket_client_sdk::gamma::types::response::Event as RawGammaEvent;

/// [`GammaEvent`] is a truncation of [`polymarket_client_sdk::gamma::types::response::Event`]
#[derive(From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GammaEvent {
    pub slug: String,
    /// NOTE: This Vec is not sorted
    pub markets: Vec<GammaMarket>,
}

impl TryFrom<RawGammaEvent> for GammaEvent {
    type Error = ();

    fn try_from(_event: RawGammaEvent) -> Result<Self, Self::Error> {
        todo!()
    }
}
