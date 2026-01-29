# Implement CacheGammaEventsListDateCascadesCommand

## Files

* src/command/cache_command.rs
* src/command/cache_download_command.rs
* src/types/output_kind.rs

## Tasks

* Implement CacheGammaEventsListDateCascadesCommand (callable as `cache gamma-events list-date-cascades`)
  * Must read the events from GAMMA_EVENTS_KEYSPACE
  * Must output the events that pass `is_date_cascade`
  * Must support `offset` and `limit` options (call `skip` and `take` on the iter)
  * Must support `kind` option (`OutputKind`)
    * Use ": " for `key_value_separator`
* Test this command by ensuring that it outputs at least one event
  * `--limit 1`
  * Timeout: 300000ms
* If the command exits successfully but doesn't output any events:
  * Run `mise db:get:gamma_events "140711"`
  * Explain to me why this event is not being output by `CacheGammaEventsListDateCascadesCommand`
