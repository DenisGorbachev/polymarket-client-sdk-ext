crate::define_cache_list_command! {
    command = CacheOrderBookSummaryResponsesListCommand,
    run_error = CacheOrderBookSummaryResponsesListCommandRunError,
    read_error = CacheOrderBookSummaryResponsesListCommandReadEntryError,
    write_error = CacheOrderBookSummaryResponsesListCommandWriteEntryError,
    keyspace_const = CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE,
}
