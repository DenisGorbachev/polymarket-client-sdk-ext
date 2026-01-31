# Use rkyv-based serialization/deserialization

## Files

* src/types/market.rs

## Tasks

* Fix src/command/cache_download_command.rs
  * Implement saving Market structs to CLOB_MARKETS_KEYSPACE
    * Create Market from MarketResponsePrecise (call Market::maybe_try_from_market_response_precise, don't save if None, do handle the error if Some)
    * Serialize the Market via rkyv
    * Save the serialized Market

## Tests

* `cargo run --quiet -- cache download --offset 0 --page-limit 3`
