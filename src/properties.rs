use crate::Property;
use polymarket_client_sdk::clob::types::response::MarketResponse;

pub type PropertyFactory<T> = fn() -> Box<dyn Property<T>>;

#[linkme::distributed_slice]
pub static MARKET_RESPONSE_PROPERTIES: [PropertyFactory<MarketResponse>] = [..];

#[doc(hidden)]
#[macro_export]
macro_rules! register_property {
    ($typ:ident, $target:ty, $slice:ident) => {
        #[linkme::distributed_slice($slice)]
        fn factory() -> Box<dyn $crate::Property<$target>> {
            Box::new($typ::default())
        }
    };
}

mod market_slug_is_unique;

pub use market_slug_is_unique::*;

mod question_id_is_none_iff_condition_id_is_none;

pub use question_id_is_none_iff_condition_id_is_none::*;

mod if_is5050outcome_then_both_tokens_are_winners;

pub use if_is5050outcome_then_both_tokens_are_winners::*;

mod tokens_len_is_two;

pub use tokens_len_is_two::*;

mod token_prices_are_probabilities;

pub use token_prices_are_probabilities::*;

mod if_any_token_is_winner_then_market_is_closed;

pub use if_any_token_is_winner_then_market_is_closed::*;

mod if_condition_id_is_none_then_tokens_are_placeholders;

pub use if_condition_id_is_none_then_tokens_are_placeholders::*;

mod if_condition_id_is_none_then_orders_are_disabled;

pub use if_condition_id_is_none_then_orders_are_disabled::*;

mod token_id_is_unique_or_zero;

pub use token_id_is_unique_or_zero::*;

mod active_xor_closed;

pub use active_xor_closed::*;

mod max_winner_token_count_is_one;

pub use max_winner_token_count_is_one::*;
