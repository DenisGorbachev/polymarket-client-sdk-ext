use crate::{Token, TokenId};
use derive_more::{Error, From, Into};
use derive_new::new;
use fmt_derive::Display;
use serde::{Deserialize, Serialize};

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

    pub fn token_ids_vec(&self) -> Vec<TokenId> {
        vec![self.left.token_id, self.right.token_id]
    }
}

#[derive(Error, Display, From, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum TokensValidationError {}
