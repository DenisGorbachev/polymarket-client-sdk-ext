pub const YES: &str = "Yes";
pub const NO: &str = "No";

/// Default page size: 20
/// Max page size: 500 (experimentally verified)
/// Must be explicitly set in order to take effect
pub const GAMMA_EVENTS_PAGE_SIZE: i32 = 500;

pub const GAMMA_EVENTS_KEYSPACE: &str = "gamma_events";
pub const CLOB_MARKET_RESPONSES_KEYSPACE: &str = "clob_market_responses";
pub const CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE: &str = "clob_order_book_summary_responses";
