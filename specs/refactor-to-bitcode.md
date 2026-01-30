# Refactor this crate to use `bitcode`

## Files

* src/command/cache_command.rs
* src/types/market.rs
* src/types/orderbook.rs
* src/command/cache_download_command.rs
* src/command/cache_gamma_events_command/cache_gamma_events_list_date_cascades_command.rs

## Goal

Make the `fjall` database reads and writes faster when serialization / deserialization is required.

## Tasks

* Run the [timing command](#timing-command)
* Refactor the CacheDownloadCommand and CacheGammaEventsListDateCascadesCommand to use `bitcode` instead of `serde_json`
  * Keep the user-facing output in JSON
* Run the [timing command](#timing-command) again (ensure that the time is lower)

## Definitions

### Timing command

`time cargo run --quiet -- cache gamma-events list-date-cascades --kind key`

## Notes

* Use latest <https://crates.io/crates/bitcode>
  * Enable the `serde` integration feature
* In your report, mention the issues that you ran into during implementation
