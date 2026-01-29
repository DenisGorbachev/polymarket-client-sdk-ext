use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use alloy::primitives::U256;
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;
use rustc_hash::FxHashSet;

#[derive(Default, Eq, PartialEq, Clone, Debug)]
pub struct TokenIdIsUniqueOrZero {
    token_ids: FxHashSet<U256>,
}

impl Property<MarketResponse> for TokenIdIsUniqueOrZero {
    fn holds(&mut self, market_response: &MarketResponse, _snapshot: &Snapshot) -> bool {
        market_response
            .tokens
            .iter()
            .all(|token| if token.token_id.is_zero() { true } else { self.token_ids.insert(token.token_id) })
    }
}

register_property!(TokenIdIsUniqueOrZero, MarketResponse, MARKET_RESPONSE_PROPERTIES);
