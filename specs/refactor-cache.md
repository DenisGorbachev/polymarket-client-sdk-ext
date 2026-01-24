# Refactor test cache

## Files

* src/types/market.rs
* src/test_helpers.rs

## Tasks

* Refactor the test cache download code (that downloads markets) into src/tests/caching_tests.rs
* Don't use `stream_data`, implement your own streaming code similar to `stream_data`
* Implement downloading orderbooks (`OrderBookSummaryResponse`) in the same code
  * Collect the `token_id` from markets page
  * Call one `Client::order_books` per markets page
* Add a `#[cfg(test)]` annotation on `mod tests` in src/lib.rs
* In src/types/market.rs and src/types/orderbook.rs, don't initiate the cache download (only assert that it exists - it should fail if cache doesn't exist)
* In src/tests/caching_tests.rs, always re-download the cache (drop the env var check related code)
* It looks like `stream!` won't be needed, and you can implement proper error handling
* Introduce `MARKET_RESPONSE_CACHE_LIMIT` env var and use it to limit the number of downloaded markets
* Test by running nextest with a filter override (see mise.toml) and a MARKET_RESPONSE_CACHE_LIMIT=200

## Notes

* These tests will not run by default due to filter in .config/nextest.toml
