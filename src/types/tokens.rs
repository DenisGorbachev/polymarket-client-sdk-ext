use crate::{Token, TokenId};
use derive_more::{From, Into};
use derive_new::new;
use polymarket_client_sdk::clob::types::response::Token as TokenRaw;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// IMPORTANT: Do not assume that `self.left.outcome == "Yes"` or `self.right.outcome == "No"`
#[derive(new, From, Into, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct Tokens {
    pub left: Token,
    pub right: Token,
}

impl Tokens {
    /// `self.left.winner` and `self.right.winner` can be `true` at the same time if `market.is_50_50_outcome == true`
    pub fn winner(&self) -> Option<Option<&Token>> {
        match (self.left.winner, self.right.winner) {
            (true, true) => None,
            (false, false) => Some(None),
            (true, false) => Some(Some(&self.left)),
            (false, true) => Some(Some(&self.right)),
        }
    }

    pub fn token_ids_tuple(&self) -> (TokenId, TokenId) {
        (self.left.token_id, self.right.token_id)
    }

    pub fn token_ids_array(&self) -> [TokenId; 2] {
        [self.left.token_id, self.right.token_id]
    }
}

// TODO: Fix error handling
#[allow(clippy::infallible_try_from)]
impl TryFrom<Vec<TokenRaw>> for Tokens {
    type Error = TryFromVecTokenRawForTokens;

    fn try_from(_value: Vec<TokenRaw>) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum TryFromVecTokenRawForTokens {}
