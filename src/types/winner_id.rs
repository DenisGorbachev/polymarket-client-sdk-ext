use crate::TokenId;
use derive_more::From;

#[derive(From, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum WinnerId {
    One(TokenId),
    Both,
}

impl WinnerId {}
