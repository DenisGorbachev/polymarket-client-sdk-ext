use crate::Property;
use polymarket_client_sdk::clob::types::response::MarketResponse;

pub fn market_response_properties() -> Vec<Box<dyn Property<MarketResponse>>> {
    vec![
        Box::new(MarketSlugIsUnique::default()),
        Box::new(QuestionIdIsNoneIffConditionIdIsNone),
        Box::new(Is5050OutcomeIffBothTokensAreWinners),
    ]
}

mod market_slug_is_unique;

pub use market_slug_is_unique::*;

mod question_id_is_none_iff_condition_id_is_none;

pub use question_id_is_none_iff_condition_id_is_none::*;

mod is_50_50_outcome_iff_both_tokens_are_winners;

pub use is_50_50_outcome_iff_both_tokens_are_winners::*;
