crate::define_cache_list_command! {
    command = CacheMarketResponsesListCommand,
    run_error = CacheMarketResponsesListCommandRunError,
    read_error = CacheMarketResponsesListCommandReadEntryError,
    write_error = CacheMarketResponsesListCommandWriteEntryError,
    keyspace_const = CLOB_MARKET_RESPONSE_KEYSPACE,
}
