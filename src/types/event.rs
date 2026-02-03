use derive_more::{From, Into};

#[derive(From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GammaEvent {
    pub slug: String,
    // /// Must be sorted by (end_date, id)
    // pub markets: Vec<Market>,
}
