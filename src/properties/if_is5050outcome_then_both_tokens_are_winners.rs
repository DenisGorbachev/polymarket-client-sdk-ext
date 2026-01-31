use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct IfIs5050OutcomeThenBothTokensAreWinners;

impl Property<MarketResponse> for IfIs5050OutcomeThenBothTokensAreWinners {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        if value.is_50_50_outcome { value.tokens.iter().all(|token| token.winner) } else { true }
    }
}

register_property!(IfIs5050OutcomeThenBothTokensAreWinners, MarketResponse, MARKET_RESPONSE_PROPERTIES);
