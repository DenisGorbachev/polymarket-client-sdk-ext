use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct IfAnyTokenIsWinnerThenMarketIsClosed;

impl Property<MarketResponse> for IfAnyTokenIsWinnerThenMarketIsClosed {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        let has_winner = value.tokens.iter().any(|token| token.winner);
        !has_winner || value.closed
    }
}

register_property!(IfAnyTokenIsWinnerThenMarketIsClosed, MarketResponse, MARKET_RESPONSE_PROPERTIES);
