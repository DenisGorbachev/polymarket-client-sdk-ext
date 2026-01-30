use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct MaxWinnerTokenCountIsOne;

impl Property<MarketResponse> for MaxWinnerTokenCountIsOne {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        let winners = value.tokens.iter().filter(|token| token.winner);
        winners.count() <= 1
    }
}

register_property!(MaxWinnerTokenCountIsOne, MarketResponse, MARKET_RESPONSE_PROPERTIES);
