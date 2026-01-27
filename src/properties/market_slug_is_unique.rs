use crate::Holds;
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;
use rustc_hash::FxHashSet;

#[derive(Default, Eq, PartialEq, Clone, Debug)]
pub struct MarketSlugIsUnique {
    slugs: FxHashSet<String>,
}

impl Holds<MarketResponse> for MarketSlugIsUnique {
    fn holds(&mut self, market_response: &MarketResponse, _snapshot: &Snapshot) -> bool {
        // returns true if the set didn't contain this value
        self.slugs.insert(market_response.market_slug.clone())
    }
}
