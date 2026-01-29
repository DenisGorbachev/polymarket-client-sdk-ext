use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct TokensLenIsTwo;

impl Property<MarketResponse> for TokensLenIsTwo {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        value.tokens.len() == 2
    }
}

register_property!(TokensLenIsTwo, MarketResponse, MARKET_RESPONSE_PROPERTIES);
