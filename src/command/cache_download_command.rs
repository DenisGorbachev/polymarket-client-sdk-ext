use crate::{CLOB_MARKET_RESPONSES_KEYSPACE, CLOB_MARKETS_KEYSPACE, CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE, ClobMarket, ClobMarketFallible, ClobMarketResponsePrecise, ClobMarketResponsePreciseFallible, ConvertOrderBookSummaryResponseToOrderbookError, DEFAULT_DB_DIR, GAMMA_EVENTS_KEYSPACE, GAMMA_EVENTS_PAGE_SIZE, GAMMA_QUERY_ASCENDING, GammaEvent, NEXT_CURSOR_STOP, NextCursor, OpenKeyspaceError, OrderBookSummaryResponsePrecise, ShouldDownloadOrderbooks, TokenId, format_debug_diff, gamma_event_raw_is_fresh, open_keyspace, progress_report_line};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use errgonomic::{DisplayAsDebug, ErrVec, handle, handle_bool, handle_iter, map_err};
use fjall::{PersistMode, SingleWriterTxDatabase, SingleWriterTxKeyspace, SingleWriterWriteTx, UserKey};
use futures::future::join_all;
use itertools::Itertools;
use polymarket_client_sdk::clob::Client as ClobClient;
use polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest;
use polymarket_client_sdk::clob::types::response::{MarketResponse, OrderBookSummaryResponse};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::EventsRequest;
use polymarket_client_sdk::gamma::types::response::Event;
use rustc_hash::FxHashSet;
use std::error::Error as StdError;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

const ORDERBOOKS_CHUNK_SIZE: usize = 500;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheDownloadCommand {
    /// A limit on the number of downloaded pages (applies to all paginated endpoints)
    #[arg(long)]
    pub page_limit: Option<NonZeroUsize>,

    /// A starting offset that overrides the cached keyspace length
    #[arg(long)]
    pub offset: Option<usize>,

    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,
}

impl CacheDownloadCommand {
    pub async fn run(self) -> Result<ExitCode, CacheDownloadCommandRunError> {
        use CacheDownloadCommandRunError::*;
        let Self {
            page_limit,
            offset,
            dir,
        } = self;
        let db = handle!(SingleWriterTxDatabase::builder(&dir).open(), OpenDatabaseFailed, dir);
        let market_response_keyspace = handle!(open_keyspace(&db, CLOB_MARKET_RESPONSES_KEYSPACE), KeyspaceOpenFailed);
        let market_keyspace = handle!(open_keyspace(&db, CLOB_MARKETS_KEYSPACE), KeyspaceOpenFailed);
        let orderbook_keyspace = handle!(open_keyspace(&db, CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE), KeyspaceOpenFailed);
        let event_keyspace = handle!(open_keyspace(&db, GAMMA_EVENTS_KEYSPACE), KeyspaceOpenFailed);
        let clob_client = ClobClient::default();
        let gamma_client = GammaClient::default();
        let page_limit = page_limit.map(NonZeroUsize::get);
        let markets_download = async {
            use CacheDownloadCommandRunError::*;
            map_err!(Self::download_market_responses(&db, &market_response_keyspace, &market_keyspace, &orderbook_keyspace, &clob_client, page_limit, offset).await, DownloadMarketResponsesFailed)
        };
        let events_download = async {
            use CacheDownloadCommandRunError::*;
            map_err!(Self::download_gamma_events(&db, &event_keyspace, &gamma_client, page_limit, offset).await, DownloadGammaEventsFailed)
        };
        let result = tokio::try_join!(markets_download, events_download);
        match result {
            Ok((_markets, _events)) => (),
            Err(error) => return Err(error),
        }
        Ok(ExitCode::SUCCESS)
    }

    async fn download_market_responses(db: &SingleWriterTxDatabase, market_response_keyspace: &SingleWriterTxKeyspace, market_keyspace: &SingleWriterTxKeyspace, orderbook_keyspace: &SingleWriterTxKeyspace, client: &ClobClient, page_limit: Option<usize>, offset: Option<usize>) -> Result<(), CacheDownloadCommandDownloadMarketResponsesError> {
        use CacheDownloadCommandDownloadMarketResponsesError::*;
        let mut offset = match offset {
            Some(offset) => offset,
            None => handle!(market_response_keyspace.as_ref().len(), MarketKeyspaceLenFailed),
        };
        let mut next_cursor: NextCursor = STANDARD.encode(offset.to_string());
        let mut market_slugs = FxHashSet::default();
        let mut page_offset: usize = 0;

        loop {
            eprintln!("{}", progress_report_line("Downloading markets", offset, None, None, page_offset, page_limit));
            let page = handle!(client.markets(Some(next_cursor.clone())).await, FetchMarketsFailed, next_cursor);
            let markets = page.data;
            let next_cursor_new = page.next_cursor;
            if markets.is_empty() {
                break;
            }
            let duplicates = Self::get_duplicates(&markets, |x| x.market_slug.clone(), &mut market_slugs).collect_vec();
            handle_bool!(!duplicates.is_empty(), DuplicatesFound, duplicates);
            let market_count = markets.len();
            let token_ids = markets
                .iter()
                .filter(|m| m.should_download_orderbooks())
                .flat_map(|market_response| market_response.tokens.iter().map(|t| t.token_id));
            let orderbooks = handle!(Self::fetch_orderbooks_for_tokens(client, token_ids).await, FetchOrderbooksForTokensFailed);
            handle!(Self::write_market_response_page_to_database(db, market_response_keyspace, market_keyspace, orderbook_keyspace, markets, orderbooks), WritePageToDatabaseFailed);
            offset = offset.saturating_add(market_count);
            page_offset = page_offset.saturating_add(1);
            next_cursor = next_cursor_new;
            if next_cursor == NEXT_CURSOR_STOP || Self::limit_reached(page_offset, page_limit) {
                break;
            }
        }
        Ok(())
    }

    async fn download_gamma_events(db: &SingleWriterTxDatabase, event_keyspace: &SingleWriterTxKeyspace, client: &GammaClient, page_limit: Option<usize>, offset: Option<usize>) -> Result<(), CacheDownloadCommandDownloadGammaEventsError> {
        use CacheDownloadCommandDownloadGammaEventsError::*;
        let mut offset = match offset {
            Some(offset) => offset,
            None => handle!(event_keyspace.as_ref().len(), EventKeyspaceLenFailed),
        };
        let mut event_slugs = FxHashSet::default();
        let mut page_offset: usize = 0;
        let page_size = GAMMA_EVENTS_PAGE_SIZE;

        loop {
            eprintln!("{}", progress_report_line("Downloading events", offset, Some(page_size), None, page_offset, page_limit));
            let request = EventsRequest::builder()
                .order(vec!["id".to_string()])
                .ascending(GAMMA_QUERY_ASCENDING)
                .limit(GAMMA_EVENTS_PAGE_SIZE as i32)
                .offset(offset as i32)
                .build();
            let events = handle!(client.events(&request).await, FetchEventsFailed, request);
            if events.is_empty() {
                break;
            }
            let event_count = events.len();
            handle!(Self::write_events_to_database(db, event_keyspace, &mut event_slugs, events), WriteEventsToDatabaseFailed);
            offset = offset.saturating_add(event_count);
            page_offset = page_offset.saturating_add(1);
            if event_count < page_size || Self::limit_reached(page_offset, page_limit) {
                break;
            }
        }
        Ok(())
    }

    async fn fetch_orderbooks_for_tokens(client: &ClobClient, token_ids: impl Iterator<Item = TokenId>) -> Result<Vec<OrderBookSummaryResponse>, CacheDownloadCommandFetchOrderbooksForTokensError> {
        use CacheDownloadCommandFetchOrderbooksForTokensError::*;
        let chunks = token_ids.chunks(ORDERBOOKS_CHUNK_SIZE);
        let futures = chunks
            .into_iter()
            .map(|chunk| Self::fetch_orderbooks_chunk(client, chunk));
        let results = join_all(futures).await;
        let orderbooks = handle_iter!(results.into_iter(), FetchOrderbooksChunkFailed);
        Ok(orderbooks.into_iter().flatten().collect())
    }

    async fn fetch_orderbooks_chunk(client: &ClobClient, token_ids: impl Iterator<Item = TokenId>) -> Result<Vec<OrderBookSummaryResponse>, CacheDownloadCommandFetchOrderbooksChunkError> {
        use CacheDownloadCommandFetchOrderbooksChunkError::*;
        let requests = token_ids
            .map(|token_id| {
                OrderBookSummaryRequest::builder()
                    .token_id(token_id)
                    .build()
            })
            .collect::<Vec<_>>();
        let orderbooks = handle!(client.order_books(&requests).await, OrderBooksFailed, requests: requests.into_boxed_slice());
        Ok(orderbooks)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_market_response_page_to_database(db: &SingleWriterTxDatabase, market_response_keyspace: &SingleWriterTxKeyspace, market_keyspace: &SingleWriterTxKeyspace, orderbook_keyspace: &SingleWriterTxKeyspace, markets: Vec<MarketResponse>, orderbooks: Vec<OrderBookSummaryResponse>) -> Result<(), CacheDownloadCommandWritePageToDatabaseError> {
        use CacheDownloadCommandWritePageToDatabaseError::*;
        let market_entries = handle_iter!(
            markets.into_iter().map(|market_response| {
                use CacheDownloadCommandMarketEntriesFromResponseError::*;
                let (_market_response, market_precise) = handle!(Self::round_trip_entry::<MarketResponse, ClobMarketResponsePrecise, ClobMarketResponsePreciseFallible>(market_response), RoundTripEntryFailed);
                let market_entry_opt = match ClobMarket::maybe_try_from_market_response_precise(market_precise.clone()) {
                    None => None,
                    Some(result) => {
                        let market = handle!(result, MarketTryFromFailed);
                        Some(market)
                    }
                };
                Ok((market_precise, market_entry_opt))
            }),
            MarketEntriesFromResponseFailed
        );
        let (market_responses, markets) = market_entries
            .into_iter()
            .fold((Vec::new(), Vec::new()), |(mut responses, mut markets), (response_entry, market_entry_opt)| {
                responses.push(response_entry);
                if let Some(market_entry) = market_entry_opt {
                    markets.push(market_entry);
                }
                (responses, markets)
            });
        let mut tx = db.write_tx();
        let _market_response_inserts = handle_iter!(Self::insert_iter(&mut tx, market_response_keyspace, market_responses, |market_response| market_response.market_slug.as_str().into(), Self::market_response_bytes), InsertMarketResponseEntriesFailed);
        let _market_inserts = handle_iter!(Self::insert_iter(&mut tx, market_keyspace, markets, |market| market.slug.as_str().into(), Self::market_bytes), InsertMarketEntriesFailed);
        let _orderbook_inserts = handle_iter!(Self::insert_iter(&mut tx, orderbook_keyspace, orderbooks, |orderbook| orderbook.asset_id.to_string().into(), Self::orderbook_bytes), InsertOrderbookEntriesFailed);
        handle!(tx.commit(), CommitTransactionFailed);
        handle!(db.persist(PersistMode::SyncAll), PersistDatabaseFailed);
        Ok(())
    }

    fn write_events_to_database(db: &SingleWriterTxDatabase, event_keyspace: &SingleWriterTxKeyspace, event_slugs: &mut FxHashSet<String>, events: Vec<Event>) -> Result<(), CacheDownloadCommandWriteEventsToDatabaseError> {
        use CacheDownloadCommandWriteEventsToDatabaseError::*;
        let event_entries = handle_iter!(
            events
                .into_iter()
                .filter(gamma_event_raw_is_fresh)
                .map(|event| {
                    use CacheDownloadCommandEventEntryFromResponseError::*;
                    let event = handle!(GammaEvent::try_from(event), TryFromFailed);
                    let event_slug = event.slug.clone();
                    Ok((event_slug, event))
                }),
            EventEntryFromResponseFailed
        );
        let duplicates = Self::get_duplicates(&event_entries, |(event_slug, _)| event_slug.clone(), event_slugs).collect_vec();
        handle_bool!(!duplicates.is_empty(), DuplicatesFound, duplicates);
        let mut tx = db.write_tx();
        let _event_inserts = handle_iter!(Self::insert_iter(&mut tx, event_keyspace, event_entries, |(event_slug, _)| event_slug.as_str().into(), |(_event_slug, event)| Self::event_bytes(event)), InsertEventEntriesFailed);
        handle!(tx.commit(), CommitTransactionFailed);
        handle!(db.persist(PersistMode::SyncAll), PersistDatabaseFailed);
        Ok(())
    }

    fn round_trip_entry<T, U, E>(input: T) -> Result<(T, U), CacheDownloadCommandRoundTripEntryError<T, E>>
    where
        T: Clone + PartialEq + core::fmt::Debug,
        U: TryFrom<T, Error = E> + Clone,
        T: From<U>,
        E: StdError + Send + Sync + 'static,
    {
        use CacheDownloadCommandRoundTripEntryError::*;
        let output = handle!(U::try_from(input.clone()), TryFromFailed, input);
        let input_round_trip = T::from(output.clone());
        handle_bool!(
            input != input_round_trip,
            RoundTripFailed,
            diff: format_debug_diff(&input, &input_round_trip, "input", "input_round_trip"),
            input,
            input_round_trip
        );
        Ok((input, output))
    }

    fn insert<T, E>(tx: &mut SingleWriterWriteTx, keyspace: &SingleWriterTxKeyspace, key: UserKey, value: T, serialize: &mut impl FnMut(T) -> Result<Vec<u8>, E>) -> Result<(), CacheDownloadCommandInsertError<E>>
    where
        E: StdError + Send + Sync + 'static,
    {
        use CacheDownloadCommandInsertError::*;
        let bytes = handle!(serialize(value), SerializeFailed, key: DisplayAsDebug::from(key));
        tx.insert(keyspace, key, bytes);
        Ok(())
    }

    fn insert_iter<T, E>(tx: &mut SingleWriterWriteTx, keyspace: &SingleWriterTxKeyspace, values: impl IntoIterator<Item = T>, mut get_key: impl FnMut(&T) -> UserKey, mut serialize: impl FnMut(T) -> Result<Vec<u8>, E>) -> impl Iterator<Item = Result<(), CacheDownloadCommandInsertError<E>>>
    where
        E: StdError + Send + Sync + 'static,
    {
        values.into_iter().map(move |value| {
            let key = get_key(&value);
            Self::insert(tx, keyspace, key, value, &mut serialize)
        })
    }

    fn get_duplicates<'a, T: 'a, I: Eq + Hash>(values: impl IntoIterator<Item = &'a T>, mut map: impl FnMut(&'a T) -> I, seen: &mut FxHashSet<I>) -> impl Iterator<Item = I> {
        values.into_iter().filter_map(move |x| {
            let input = map(x);
            if seen.contains(&input) {
                Some(input)
            } else {
                seen.insert(input);
                None
            }
        })
    }

    fn market_response_bytes(market: ClobMarketResponsePrecise) -> Result<Vec<u8>, CacheDownloadCommandMarketResponseBytesError> {
        use CacheDownloadCommandMarketResponseBytesError::*;
        let bytes = handle!(rkyv::to_bytes::<rkyv::rancor::Error>(&market), SerializeFailed, market);
        Ok(bytes.into_vec())
    }

    fn market_bytes(market: ClobMarket) -> Result<Vec<u8>, CacheDownloadCommandMarketBytesError> {
        use CacheDownloadCommandMarketBytesError::*;
        let bytes = handle!(rkyv::to_bytes::<rkyv::rancor::Error>(&market), SerializeFailed, market);
        Ok(bytes.into_vec())
    }

    fn orderbook_bytes(orderbook: OrderBookSummaryResponse) -> Result<Vec<u8>, CacheDownloadCommandOrderbookBytesError> {
        use CacheDownloadCommandOrderbookBytesError::*;
        let (_orderbook, orderbook_precise) = handle!(Self::round_trip_entry::<OrderBookSummaryResponse, OrderBookSummaryResponsePrecise, ConvertOrderBookSummaryResponseToOrderbookError>(orderbook), RoundTripEntryFailed);
        let bytes = handle!(rkyv::to_bytes::<rkyv::rancor::Error>(&orderbook_precise), SerializeFailed, orderbook: Box::new(orderbook_precise));
        Ok(bytes.into_vec())
    }

    fn event_bytes(event: GammaEvent) -> Result<Vec<u8>, CacheDownloadCommandEventBytesError> {
        use CacheDownloadCommandEventBytesError::*;
        let bytes = handle!(rkyv::to_bytes::<rkyv::rancor::Error>(&event), SerializeFailed, event);
        Ok(bytes.into_vec())
    }

    fn limit_reached(offset: usize, limit: Option<usize>) -> bool {
        match limit {
            Some(limit) => offset >= limit,
            None => false,
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandRunError {
    #[error("failed to open database at '{dir}'")]
    OpenDatabaseFailed { source: fjall::Error, dir: PathBuf },
    #[error("failed to open keyspace")]
    KeyspaceOpenFailed { source: OpenKeyspaceError },
    #[error("failed to download market responses")]
    DownloadMarketResponsesFailed { source: CacheDownloadCommandDownloadMarketResponsesError },
    #[error("failed to download gamma events")]
    DownloadGammaEventsFailed { source: CacheDownloadCommandDownloadGammaEventsError },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandDownloadMarketResponsesError {
    #[error("failed to read market keyspace length")]
    MarketKeyspaceLenFailed { source: fjall::Error },
    #[error("failed to fetch markets page with cursor '{next_cursor}'")]
    FetchMarketsFailed { source: polymarket_client_sdk::error::Error, next_cursor: NextCursor },
    #[error("found {len} duplicates", len = duplicates.len())]
    DuplicatesFound { duplicates: Vec<String> },
    #[error("failed to fetch order books")]
    FetchOrderbooksForTokensFailed { source: CacheDownloadCommandFetchOrderbooksForTokensError },
    #[error("failed to persist page to database")]
    WritePageToDatabaseFailed { source: CacheDownloadCommandWritePageToDatabaseError },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandDownloadGammaEventsError {
    #[error("failed to read event keyspace length")]
    EventKeyspaceLenFailed { source: fjall::Error },
    #[error("failed to fetch gamma events page")]
    FetchEventsFailed { source: polymarket_client_sdk::error::Error, request: Box<EventsRequest> },
    #[error("failed to convert event count '{count}' to offset")]
    EventCountConversionFailed { source: core::num::TryFromIntError, count: usize },
    #[error("failed to persist events to database")]
    WriteEventsToDatabaseFailed { source: CacheDownloadCommandWriteEventsToDatabaseError },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandEventEntryFromResponseError {
    #[error("failed to convert gamma event response")]
    TryFromFailed { source: Box<crate::ConvertGammaEventRawToGammaEventError> },
    #[error("event response has duplicate event slug '{event_slug}'")]
    EventSlugDuplicateInvalid { event_slug: String },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandFetchOrderbooksForTokensError {
    #[error("failed to fetch order books for {len} chunks", len = source.len())]
    FetchOrderbooksChunkFailed { source: ErrVec<CacheDownloadCommandFetchOrderbooksChunkError> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandFetchOrderbooksChunkError {
    #[error("failed to fetch order books for chunk")]
    OrderBooksFailed { source: polymarket_client_sdk::error::Error, requests: Box<[OrderBookSummaryRequest]> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandWritePageToDatabaseError {
    #[error("failed to parse {len} market responses", len = source.len())]
    MarketEntriesFromResponseFailed { source: ErrVec<CacheDownloadCommandMarketEntriesFromResponseError> },
    #[error("failed to insert market response entries")]
    InsertMarketResponseEntriesFailed { source: ErrVec<CacheDownloadCommandInsertError<CacheDownloadCommandMarketResponseBytesError>> },
    #[error("failed to insert market entries")]
    InsertMarketEntriesFailed { source: ErrVec<CacheDownloadCommandInsertError<CacheDownloadCommandMarketBytesError>> },
    #[error("failed to insert order book entries")]
    InsertOrderbookEntriesFailed { source: ErrVec<CacheDownloadCommandInsertError<CacheDownloadCommandOrderbookBytesError>> },
    #[error("failed to commit database transaction")]
    CommitTransactionFailed { source: fjall::Error },
    #[error("failed to persist database changes")]
    PersistDatabaseFailed { source: fjall::Error },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandWriteEventsToDatabaseError {
    #[error("failed to parse {len} event responses", len = source.len())]
    EventEntryFromResponseFailed { source: ErrVec<CacheDownloadCommandEventEntryFromResponseError> },
    #[error("found {len} duplicates", len = duplicates.len())]
    DuplicatesFound { duplicates: Vec<String> },
    #[error("failed to insert event entries")]
    InsertEventEntriesFailed { source: ErrVec<CacheDownloadCommandInsertError<CacheDownloadCommandEventBytesError>> },
    #[error("failed to commit database transaction")]
    CommitTransactionFailed { source: fjall::Error },
    #[error("failed to persist database changes")]
    PersistDatabaseFailed { source: fjall::Error },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandRoundTripEntryError<T, E>
where
    T: core::fmt::Debug,
    E: StdError + Send + Sync + 'static,
{
    #[error("failed to convert cache entry")]
    TryFromFailed { source: E, input: Box<T> },
    #[error("round-tripped cache entry does not match original: '{diff}'")]
    RoundTripFailed { input: Box<T>, input_round_trip: Box<T>, diff: String },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandMarketEntriesFromResponseError {
    #[error("failed to round-trip market response")]
    RoundTripEntryFailed { source: Box<CacheDownloadCommandRoundTripEntryError<MarketResponse, ClobMarketResponsePreciseFallible>> },
    #[error("failed to convert market response to market")]
    MarketTryFromFailed { source: Box<ClobMarketFallible> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandInsertError<E>
where
    E: StdError + Send + Sync + 'static,
{
    #[error("failed to serialize entry for key '{key}'")]
    SerializeFailed { source: E, key: DisplayAsDebug<UserKey> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandMarketResponseBytesError {
    #[error("failed to serialize market response")]
    SerializeFailed { source: rkyv::rancor::Error, market: Box<ClobMarketResponsePrecise> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandMarketBytesError {
    #[error("failed to serialize market")]
    SerializeFailed { source: rkyv::rancor::Error, market: Box<ClobMarket> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandOrderbookBytesError {
    #[error("failed to round-trip order book summary response")]
    RoundTripEntryFailed { source: CacheDownloadCommandRoundTripEntryError<OrderBookSummaryResponse, ConvertOrderBookSummaryResponseToOrderbookError> },
    #[error("failed to serialize order book summary")]
    SerializeFailed { source: rkyv::rancor::Error, orderbook: Box<OrderBookSummaryResponsePrecise> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandEventBytesError {
    #[error("failed to serialize event response")]
    SerializeFailed { source: rkyv::rancor::Error, event: Box<GammaEvent> },
}
