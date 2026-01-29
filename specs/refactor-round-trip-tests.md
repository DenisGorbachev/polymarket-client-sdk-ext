# Refactor round-trip tests into a separate command

## Files

* src/types/market.rs
* src/types/orderbook.rs

## Tasks

* Implement a `CacheTestCommand`
  * Load the markets and orderbooks from the cache db (see src/command/cache_check_command.rs)
  * Use iterators instead of streams
    * Use exit_iterator_of_results_print_first
  * Don't duplicate the code, write generic code that works for markets and orderbooks
  * Parallelize the testing if possible
    * Use `par_iter` from `rayon` if possible
* Remove the `must_round_trip_cache` tests
