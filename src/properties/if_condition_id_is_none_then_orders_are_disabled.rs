use crate::{MARKET_RESPONSE_PROPERTIES, Property};
use fjall::Snapshot;
use polymarket_client_sdk::clob::types::response::MarketResponse;

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct IfConditionIdIsNoneThenOrdersAreDisabled;

impl Property<MarketResponse> for IfConditionIdIsNoneThenOrdersAreDisabled {
    fn holds(&mut self, value: &MarketResponse, _snapshot: &Snapshot) -> bool {
        if value.condition_id.is_none() {
            !value.enable_order_book && !value.accepting_orders && value.accepting_order_timestamp.is_none()
        } else {
            true
        }
    }
}

register_property!(IfConditionIdIsNoneThenOrdersAreDisabled, MarketResponse, MARKET_RESPONSE_PROPERTIES);
