* Remove the use of BoolishValueParser, write your own parsing code
* Move the generic functions (not related specifically to markets) to src/test_helpers.rs
* Don't use MARKET_RESPONSE_CACHE_PATH_ENV, only DEFAULT_MARKET_RESPONSE_CACHE_PATH
* Use `const DEFAULT_MARKET_RESPONSE_CACHE_PATH: &str = "cache.local/market_response.all.jsonl";`
* Implement proper error handling for `refresh_market_response_cache`
* Remove custom downloading code, create a `Client` and use `Client::markets` (`use polymarket_client_sdk::clob::Client;`)
* Use `https://crates.io/crates/async-jsonl` instead of your custom JSONL parsing code
* Write the code in a way that doesn't wait until the full cache has been downloaded
  * Return an `impl Stream<Item = MarketResponse>` from the function that refreshes the cache
* In `market_response_cache_temp_path`, consider using `cache_path.with_extension`
* Rename `market_response_cache_temp_path` to `to_tmp_path`
