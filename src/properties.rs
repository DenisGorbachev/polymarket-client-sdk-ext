use crate::Property;
use polymarket_client_sdk::clob::types::response::MarketResponse;

pub fn market_response_properties() -> Vec<Box<dyn Property<MarketResponse>>> {
    vec![
        Box::new(MarketSlugIsUnique::default()),
        Box::new(QuestionIdIsNoneIffConditionIdIsNone),
    ]
}

mod market_slug_is_unique;

pub use market_slug_is_unique::*;

mod some_markets_are_test_markets;

pub use some_markets_are_test_markets::*;
