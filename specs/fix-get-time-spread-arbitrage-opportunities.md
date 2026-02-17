# Fix get_time_spread_arbitrage_opportunities

* Remove Option from the return type (returning empty vec is enough)
* Implement proper error handling for is_inverted_pricing instead of asserts
