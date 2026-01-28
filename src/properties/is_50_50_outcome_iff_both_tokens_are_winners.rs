use crate::Property;
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct Is5050OutcomeIffBothTokensAreWinners;

impl Property<MarketResponse> for Is5050OutcomeIffBothTokensAreWinners {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        value.is_50_50_outcome == value.tokens.iter().all(|token| token.winner)
    }
}
