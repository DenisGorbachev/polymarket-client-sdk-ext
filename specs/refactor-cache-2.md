# Refactor must_refresh_cache into a CLI command

## Files

* src/tests/caching_tests.rs

## Tasks

* Implement `CacheDownloadCommand` according to CLI guidelines
  * Requirements:
    * Must have a `market_response_page_limit` parameter
    * Must have a `dir` parameter (the path to the database dir) (default: ".cache/db")
    * Must have a `market_response_keyspace` parameter (default: "clob_market_responses")
    * Must have a `order_book_summary_response_keyspace` parameter (default: "clob_order_book_summary_responses")
    * Must use a `fjall` database with `SingleWriterTxDatabase`
      * Use `condition_id` as the primary key for market
      * Use `token_id` as the primary key for order book
    * Must be able to resume the aborted download
      * Commit the transaction and sync to disk after each market page (full page with order books)
      * If the cursor is actually a base64-encoded offset:
        * Then: create a cursor for resuming just from the count of already downloaded markets
        * Else: store the cursor in a separate keyspace
        * Notes:
          * There is a line `const TERMINAL_CURSOR: &str = "LTE="; // base64("-1");` in `polymarket_client_sdk` external crate
    * Must implement proper error handling instead of `match ... return Err`
    * Must call `order_books` with chunks of 500 token_ids (this is the limit of the API endpoint)
      * Process the calls for the chunks in parallel
  * Notes:
    * Move the code from src/tests/caching_tests.rs
    * `fetch_orderbooks_for_tokens` can be simplified if you call it with 500 token_ids max (no need for ranges and `is_payload_limit_error`)
* Remove src/tests.rs

## Crates

* <https://crates.io/crates/fjall>
