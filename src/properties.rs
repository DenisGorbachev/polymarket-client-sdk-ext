use crate::Property;
use polymarket_client_sdk::clob::types::response::MarketResponse;

pub fn market_response_properties() -> Vec<Box<dyn Property<MarketResponse>>> {
    vec![
        Box::new(MarketSlugIsUnique::default()),
        Box::new(QuestionIdIsNoneIffConditionIdIsNone),
        Box::new(IfIs5050OutcomeThenBothTokensAreLosers),
    ]
}

mod market_slug_is_unique;

pub use market_slug_is_unique::*;

mod question_id_is_none_iff_condition_id_is_none;

pub use question_id_is_none_iff_condition_id_is_none::*;

mod if_is_50_50_outcome_then_both_tokens_are_losers;

pub use if_is_50_50_outcome_then_both_tokens_are_losers::*;
