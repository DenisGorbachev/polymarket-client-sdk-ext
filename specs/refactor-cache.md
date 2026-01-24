# Refactor test cache

## Tasks

* Refactor the test cache download code (that downloads markets and orderbooks) into src/tests/caching_tests.rs
* Add a `#[cfg(test)]` annotation on `mod tests` in src/lib.rs
* In src/types/market.rs and src/types/orderbook.rs, don't initiate the cache download (only assert that it exists - it should fail if cache doesn't exist)
* In src/tests/caching_tests.rs, always re-download the cache (drop the env var check related code)
* It looks like `stream!` won't be needed, and you can implement proper error handling
* Introduce `MARKET_RESPONSE_CACHE_LIMIT` env var and use it to limit the number of downloaded markets
* Test by running nextest with a filter override (see mise.toml) and a MARKET_RESPONSE_CACHE_LIMIT=200

## Notes

* These tests will not run by default due to filter in .config/nextest.toml
