use crate::{Token, TokenId, WinnerId};
use derive_more::{From, Into};
use derive_new::new;
use errgonomic::{handle_bool, handle_opt};
use polymarket_client_sdk::clob::types::response::Token as TokenRaw;
use thiserror::Error;

/// IMPORTANT: Do not assume that `self.left.outcome == "Yes"` or `self.right.outcome == "No"`
#[derive(new, From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct Tokens {
    pub left: Token,
    pub right: Token,
}

impl Tokens {
    /// `self.left.winner` and `self.right.winner` can be `true` at the same time if `market.is_50_50_outcome == true` (verified with [`crate::IfIs5050OutcomeThenBothTokensAreWinners`])
    /// This function returns `Some(None)` if both tokens are winners
    pub fn winner_id(&self) -> Option<WinnerId> {
        use WinnerId::*;
        match (self.left.winner, self.right.winner) {
            (false, false) => None,
            (true, false) => Some(One(self.left.token_id)),
            (false, true) => Some(One(self.right.token_id)),
            (true, true) => Some(Both),
        }
    }

    pub fn token_ids_tuple(&self) -> (TokenId, TokenId) {
        (self.left.token_id, self.right.token_id)
    }

    pub fn token_ids_array(&self) -> [TokenId; 2] {
        [self.left.token_id, self.right.token_id]
    }
}

impl TryFrom<Vec<TokenRaw>> for Tokens {
    type Error = ConvertVecTokenRawToTokensError;

    fn try_from(tokens: Vec<TokenRaw>) -> Result<Self, Self::Error> {
        use ConvertVecTokenRawToTokensError::*;
        let tokens_len = tokens.len();
        handle_bool!(tokens_len != 2, TokensLengthInvalid, tokens_len);
        let mut tokens_iter = tokens.into_iter();
        let left = handle_opt!(tokens_iter.next(), TokensLengthInvalid, tokens_len);
        let right = handle_opt!(tokens_iter.next(), TokensLengthInvalid, tokens_len);
        let TokenRaw {
            token_id,
            outcome,
            price,
            winner,
            ..
        } = left;
        let left = Token::new(token_id, outcome, price, winner);
        let TokenRaw {
            token_id,
            outcome,
            price,
            winner,
            ..
        } = right;
        let right = Token::new(token_id, outcome, price, winner);
        Ok(Self {
            left,
            right,
        })
    }
}

impl From<Tokens> for Vec<TokenRaw> {
    fn from(tokens: Tokens) -> Self {
        let Tokens {
            left,
            right,
        } = tokens;
        [left, right]
            .into_iter()
            .map(|token| {
                let Token {
                    token_id,
                    outcome,
                    price,
                    winner,
                } = token;
                TokenRaw::builder()
                    .token_id(token_id)
                    .outcome(outcome)
                    .price(price)
                    .winner(winner)
                    .build()
            })
            .collect()
    }
}

#[derive(Error, Copy, Clone, Debug)]
pub enum ConvertVecTokenRawToTokensError {
    #[error("expected 2 tokens, got '{tokens_len}'")]
    TokensLengthInvalid { tokens_len: usize },
}
