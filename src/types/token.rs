use crate::{Amount, RkyvDecimal, TokenId};
use derive_more::{From, Into};
use derive_new::new;

#[derive(new, From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Token {
    /// Examples: `"21742633143463906290569050155826241533067272736897614950488156847949938836455"`
    pub token_id: TokenId,
    /// Examples: `"Yes"`, `"No"`
    pub outcome: String,
    /// Amount of nominal units of the quote currency (e.g. USDC)
    /// Examples: `0.5845`
    #[rkyv(with = RkyvDecimal)]
    pub price: Amount,
    /// Examples: `true`, `false`
    pub winner: bool,
}

impl Token {}
