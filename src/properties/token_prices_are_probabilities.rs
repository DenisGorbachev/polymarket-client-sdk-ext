use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;
use rust_decimal::Decimal;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct TokenPricesAreBetweenZeroAndOne;

impl Property<MarketResponse> for TokenPricesAreBetweenZeroAndOne {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        value
            .tokens
            .iter()
            .all(|token| token.price >= Decimal::ZERO && token.price <= Decimal::ONE)
    }
}

register_property!(TokenPricesAreBetweenZeroAndOne, MarketResponse, MARKET_RESPONSE_PROPERTIES);
