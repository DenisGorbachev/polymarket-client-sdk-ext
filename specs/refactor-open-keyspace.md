# Refactor db.keyspace calls

## Files

* src/command/cache_test_command.rs
* src/command/cache_download_command.rs
* src/command/cache_check_command.rs

## Tasks

* Refactor the calls to db.keyspace into a separate fn open_keyspace in src/functions/open_keyspace.rs
  * Return an error with keyspace: &'static str

Example:

```text
let keyspace = handle!(
    db.keyspace(CLOB_MARKET_RESPONSES_KEYSPACE, KeyspaceCreateOptions::default),
    OpenMarketKeyspaceFailed,
    keyspace: CLOB_MARKET_RESPONSES_KEYSPACE.to_string()
);
```
