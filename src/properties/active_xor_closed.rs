use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct ActiveXorClosed;

impl Property<MarketResponse> for ActiveXorClosed {
    fn holds(&mut self, market_response: &MarketResponse, _snapshot: &Snapshot) -> bool {
        market_response.active != market_response.closed
    }
}

register_property!(ActiveXorClosed, MarketResponse, MARKET_RESPONSE_PROPERTIES);
