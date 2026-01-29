# Find more properties

## Files

* src/command/cache_check_command.rs
* src/properties.rs
* All files in src/properties folder
* All files in .mise/tasks/db folder

## Tasks

* Explore Polymarket docs in .agents/docs/docs.polymarket.com (you already have the full list)
* Explore the actual data by running `mise run db:list:clob_market_responses --offset 100 --limit 100` (you can use any offset and limit)
* Make a list of at most 5 hypotheses regarding the properties of MarketResponse
* Implement these properties in src/properties
* Run `mise run db:check` to verify these properties (timeout: 1800000 ms (30 mins))
* Keep the properties even if they have violations
