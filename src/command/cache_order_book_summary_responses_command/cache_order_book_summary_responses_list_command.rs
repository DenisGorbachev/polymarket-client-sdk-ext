crate::define_cache_list_command! {
    command = CacheOrderBookSummaryResponsesListCommand,
    run_error = CacheOrderBookSummaryResponsesListCommandRunError,
    process_error = CacheOrderBookSummaryResponsesListCommandProcessEntryError,
    keyspace_const = CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE,
}
