use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use alloy::primitives::U256;
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;
use rust_decimal::Decimal;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct IfConditionIdIsNoneThenTokensArePlaceholders;

impl Property<MarketResponse> for IfConditionIdIsNoneThenTokensArePlaceholders {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        if value.condition_id.is_none() {
            value
                .tokens
                .iter()
                .all(|token| token.token_id == U256::ZERO && token.outcome.is_empty() && token.price == Decimal::ZERO && !token.winner)
        } else {
            true
        }
    }
}

register_property!(IfConditionIdIsNoneThenTokensArePlaceholders, MarketResponse, MARKET_RESPONSE_PROPERTIES);
