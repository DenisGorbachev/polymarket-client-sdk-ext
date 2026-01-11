use crate::{Side, TokenId, serialize_as_decimal};
use derive_more::{From, Into};
use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(new, Serialize, Deserialize, From, Into, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct BookParams {
    #[serde(serialize_with = "serialize_as_decimal")]
    token_id: TokenId,
    #[serde(skip_serializing_if = "Option::is_none")]
    side: Option<Side>,
}

impl BookParams {}

impl From<TokenId> for BookParams {
    fn from(token_id: TokenId) -> Self {
        Self {
            token_id,
            side: None,
        }
    }
}
