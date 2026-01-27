# Refactor Cache* commands

## Files

* src/command/cache_command.rs

## Tasks

* Refactor define_cache_list_command into a normal Rust type `CacheListCommand` (not macro)
  * Add `keyspace` required positional parameter
  * Add `offset` parameter (use it to call the `skip` method on iter)
* Remove `CacheMarketResponsesCommand` and `CacheOrderBookSummaryResponsesCommand`
* Add `CacheGetCommand`
  * Requirements:
    * Must have a `keyspace` required positional parameter
    * Must have a `key` positional parameter

## Tests

Run the new commands and ensure that they output some data in JSON format.
