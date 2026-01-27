crate::define_cache_list_command! {
    command = CacheMarketResponsesListCommand,
    run_error = CacheMarketResponsesListCommandRunError,
    process_error = CacheMarketResponsesListCommandProcessEntryError,
    keyspace_const = CLOB_MARKET_RESPONSE_KEYSPACE,
}
