# Refactor DownloadCacheCommand

## Files

* src/command/cache_download_command.rs

## Tasks

* Perform serialization right before tx.insert
* Use the generic `fn round_trip_entry` to test that market_response round-trips with market_response_precise
* Remove `fn serialize_market_entry`, `fn serialize_orderbook_entry`, `fn serialize_event_entry`
* Introduce `fn insert` that would serialize and insert (pass the `key` to this function as well)
* Introduce `fn insert_iter` with `get_key: impl FnMut(&T) -> UserKey`
* Use `token_ids: impl Iterator<Item = TokenId>` instead of `token_ids: impl Iterator<Item = &TokenId>` (note that TokenId is Copy)
