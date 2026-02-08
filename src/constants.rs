use std::sync::LazyLock;

pub const YES: &str = "Yes";
pub const NO: &str = "No";

/// Default page size: 20
/// Max page size: 500 (experimentally verified)
/// Must be explicitly set in order to take effect
pub const GAMMA_EVENTS_PAGE_SIZE: usize = 500;

pub const GAMMA_EVENTS_KEYSPACE: &str = "GammaEvent";
pub const CLOB_MARKET_RESPONSES_KEYSPACE: &str = "ClobMarketResponsePrecise";
pub const CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE: &str = "OrderBookSummaryResponsePrecise";

/// The keyspace for [`Market`](crate::ClobMarket)
pub const CLOB_MARKETS_KEYSPACE: &str = "ClobMarket";

// /// The keyspace for [`OrderBook`](crate::OrderBook)
// pub const CLOB_ORDER_BOOKS_KEYSPACE: &str = "clob_order_books";

/// Important: some markets have non-boolean outcomes (for example: ["Western Carolina vs. UNC Greensboro"](https://gamma-api.polymarket.com/markets/522329))
pub static BOOLEAN_OUTCOMES: LazyLock<Vec<String>> = LazyLock::new(|| vec!["Yes".to_string(), "No".to_string()]);
