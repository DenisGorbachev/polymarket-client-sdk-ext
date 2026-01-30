# Implement `cache gamma-events monitor-date-cascades` command

## Files

- src/command/cache_command.rs
- src/command/cache_download_command.rs
- src/ext/polymarket_client_sdk/gamma/types/event.rs
- src/constants.rs

## Tasks

- Implement `cache gamma-events monitor-date-cascades` command
  - Must read the events from GAMMA_EVENTS_KEYSPACE
  - Must build a Vec of ids of the events that pass `is_date_cascade`
  - Must chunk the vec of ids by `GAMMA_EVENTS_PAGE_SIZE`
  - Must loop infinitely:
    - For each chunk:
      - Fetch the updated events via `GammaClient::events`
        - Provide the chunk as `id` field of `EventRequest`
      - Save the events to GAMMA_EVENTS_KEYSPACE
      - Output the len of received events vec
