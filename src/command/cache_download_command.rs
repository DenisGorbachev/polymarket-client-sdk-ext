use crate::{CLOB_MARKET_RESPONSES_KEYSPACE, CLOB_MARKETS_KEYSPACE, CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE, ConvertMarketResponseToMarketError, ConvertOrderBookSummaryResponseToOrderbookError, DEFAULT_DB_DIR, GAMMA_EVENTS_KEYSPACE, GAMMA_EVENTS_PAGE_SIZE, Market, MarketFallible, MarketResponsePrecise, NEXT_CURSOR_STOP, NextCursor, OpenKeyspaceError, OrderBookSummaryResponsePrecise, ShouldDownloadOrderbooks, TokenId, format_debug_diff, open_keyspace, progress_report_line};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use errgonomic::{ErrVec, handle, handle_bool, handle_iter, handle_opt, map_err};
use fjall::{PersistMode, SingleWriterTxDatabase, SingleWriterTxKeyspace};
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
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

const ORDERBOOKS_CHUNK_SIZE: usize = 500;
type MarketResponseCacheEntry = (String, Vec<u8>);
type MarketCacheEntry = (String, Vec<u8>);
type MarketCacheEntries = (MarketResponseCacheEntry, Option<MarketCacheEntry>);

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
            let market_count = markets.len();
            let market_entries = handle_iter!(
                markets
                    .into_iter()
                    .map(|market| Self::market_entry_from_response(market, &mut market_slugs)),
                MarketEntryFromResponseFailed
            );
            let token_ids = market_entries
                .iter()
                .flat_map(|(_, _, token_ids)| token_ids.iter());
            let orderbooks = handle!(Self::fetch_orderbooks_for_tokens(client, token_ids).await, FetchOrderbooksForTokensFailed);
            let markets_to_store = market_entries
                .into_iter()
                .map(|(market_slug, market, _)| (market_slug, market))
                .collect::<Vec<_>>();
            handle!(Self::write_market_response_page_to_database(db, market_response_keyspace, market_keyspace, orderbook_keyspace, markets_to_store, orderbooks), WritePageToDatabaseFailed);
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
                .ascending(true)
                .limit(GAMMA_EVENTS_PAGE_SIZE as i32)
                .offset(offset as i32)
                .build();
            let events = handle!(client.events(&request).await, FetchEventsFailed, request: Box::new(request));
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

    fn market_entry_from_response(market: MarketResponse, market_slugs: &mut FxHashSet<String>) -> Result<(String, MarketResponse, Vec<TokenId>), CacheDownloadCommandMarketEntryFromResponseError> {
        use CacheDownloadCommandMarketEntryFromResponseError::*;
        let market_slug = market.market_slug.clone();
        handle_bool!(market_slug.trim().is_empty(), MarketSlugInvalid, market: Box::new(market));
        let is_duplicate = !market_slugs.insert(market_slug.clone());
        handle_bool!(is_duplicate, MarketSlugDuplicateInvalid, market_slug);
        let token_ids = if market.should_download_orderbooks() {
            market
                .tokens
                .iter()
                .map(|token| token.token_id)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        Ok((market_slug, market, token_ids))
    }

    fn event_entry_from_response(event: Event, event_slugs: &mut FxHashSet<String>) -> Result<(String, Event), CacheDownloadCommandEventEntryFromResponseError> {
        use CacheDownloadCommandEventEntryFromResponseError::*;
        let event_id = event.id.clone();
        handle_bool!(event_id.trim().is_empty(), EventIdInvalid, event: Box::new(event));
        let event_slug = handle_opt!(event.slug.clone(), EventSlugMissingInvalid, event: Box::new(event));
        handle_bool!(event_slug.trim().is_empty(), EventSlugInvalid, event_slug);
        let is_duplicate = !event_slugs.insert(event_slug.clone());
        handle_bool!(is_duplicate, EventSlugDuplicateInvalid, event_slug);
        Ok((event_id, event))
    }

    async fn fetch_orderbooks_for_tokens(client: &ClobClient, token_ids: impl Iterator<Item = &TokenId>) -> Result<Vec<OrderBookSummaryResponse>, CacheDownloadCommandFetchOrderbooksForTokensError> {
        use CacheDownloadCommandFetchOrderbooksForTokensError::*;
        let chunks = token_ids.chunks(ORDERBOOKS_CHUNK_SIZE);
        let futures = chunks
            .into_iter()
            .map(|chunk| Self::fetch_orderbooks_chunk(client, chunk));
        let results = join_all(futures).await;
        let orderbooks = handle_iter!(results.into_iter(), FetchOrderbooksChunkFailed);
        Ok(orderbooks.into_iter().flatten().collect())
    }

    async fn fetch_orderbooks_chunk(client: &ClobClient, token_ids: impl Iterator<Item = &TokenId>) -> Result<Vec<OrderBookSummaryResponse>, CacheDownloadCommandFetchOrderbooksChunkError> {
        use CacheDownloadCommandFetchOrderbooksChunkError::*;
        let requests = token_ids
            .map(|token_id| {
                OrderBookSummaryRequest::builder()
                    .token_id(*token_id)
                    .build()
            })
            .collect::<Vec<_>>();
        let orderbooks = handle!(client.order_books(&requests).await, OrderBooksFailed, requests: requests.into_boxed_slice());
        Ok(orderbooks)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_market_response_page_to_database(db: &SingleWriterTxDatabase, market_response_keyspace: &SingleWriterTxKeyspace, market_keyspace: &SingleWriterTxKeyspace, orderbook_keyspace: &SingleWriterTxKeyspace, markets: Vec<(String, MarketResponse)>, orderbooks: Vec<OrderBookSummaryResponse>) -> Result<(), CacheDownloadCommandWritePageToDatabaseError> {
        use CacheDownloadCommandWritePageToDatabaseError::*;
        let serialized_market_entries = handle_iter!(markets.into_iter().map(Self::serialize_market_entry), SerializeMarketEntryFailed);
        let (serialized_market_responses, serialized_markets) = serialized_market_entries
            .into_iter()
            .fold((Vec::new(), Vec::new()), |(mut responses, mut markets), (response_entry, market_entry_opt)| {
                responses.push(response_entry);
                if let Some(market_entry) = market_entry_opt {
                    markets.push(market_entry);
                }
                (responses, markets)
            });
        let serialized_orderbooks = handle_iter!(orderbooks.into_iter().map(Self::serialize_orderbook_entry), SerializeOrderbookEntryFailed);
        let mut tx = db.write_tx();
        let _market_response_inserts = handle_iter!(
            serialized_market_responses
                .into_iter()
                .map(|(market_slug, bytes)| {
                    tx.insert(market_response_keyspace, market_slug, bytes);
                    Ok(())
                }),
            InsertMarketResponseEntriesFailed
        );
        let _market_inserts = handle_iter!(
            serialized_markets.into_iter().map(|(market_key, bytes)| {
                tx.insert(market_keyspace, market_key, bytes);
                Ok(())
            }),
            InsertMarketEntriesFailed
        );
        let _orderbook_inserts = handle_iter!(
            serialized_orderbooks.into_iter().map(|(token_id, bytes)| {
                tx.insert(orderbook_keyspace, token_id.to_string(), bytes);
                Ok(())
            }),
            InsertOrderbookEntriesFailed
        );
        handle!(tx.commit(), CommitTransactionFailed);
        handle!(db.persist(PersistMode::SyncAll), PersistDatabaseFailed);
        Ok(())
    }

    fn write_events_to_database(db: &SingleWriterTxDatabase, event_keyspace: &SingleWriterTxKeyspace, event_slugs: &mut FxHashSet<String>, events: Vec<Event>) -> Result<(), CacheDownloadCommandWriteEventsToDatabaseError> {
        use CacheDownloadCommandWriteEventsToDatabaseError::*;
        let event_entries = handle_iter!(
            events
                .into_iter()
                .map(|event| Self::event_entry_from_response(event, event_slugs)),
            EventEntryFromResponseFailed
        );
        let serialized_events = handle_iter!(event_entries.into_iter().map(Self::serialize_event_entry), SerializeEventEntryFailed);
        let mut tx = db.write_tx();
        let _event_inserts = handle_iter!(
            serialized_events.into_iter().map(|(event_id, bytes)| {
                tx.insert(event_keyspace, event_id, bytes);
                Ok(())
            }),
            InsertEventEntriesFailed
        );
        handle!(tx.commit(), CommitTransactionFailed);
        handle!(db.persist(PersistMode::SyncAll), PersistDatabaseFailed);
        Ok(())
    }

    fn round_trip_entry<T, U, E>(input: T) -> Result<T, CacheDownloadCommandRoundTripEntryError<T, E>>
    where
        T: Clone + PartialEq + core::fmt::Debug,
        U: TryFrom<T, Error = E>,
        T: From<U>,
        E: StdError + Send + Sync + 'static,
    {
        use CacheDownloadCommandRoundTripEntryError::*;
        let output = handle!(U::try_from(input.clone()), TryFromFailed, input: Box::new(input));
        let input_round_trip = T::from(output);
        handle_bool!(
            input != input_round_trip,
            RoundTripFailed,
            diff: format_debug_diff(&input, &input_round_trip, "input", "input_round_trip"),
            input: Box::new(input),
            input_round_trip: Box::new(input_round_trip)
        );
        Ok(input)
    }

    // fn insert_entry(tx: &mut SingleWriterWriteTx, keyspace: &SingleWriterTxKeyspace, key: String, value: Vec<u8>) -> Result<(), CacheDownloadCommandInsertEntryError> {
    //     use CacheDownloadCommandInsertEntryError::*;
    //     let exists = handle!(tx.contains_key(keyspace, &key), ContainsKeyFailed, key, value);
    //     handle_bool!(exists, KeyAlreadyExists, key, value);
    //     tx.insert(keyspace, key, value);
    //     Ok(())
    // }

    fn serialize_market_entry((market_slug, market): (String, MarketResponse)) -> Result<MarketCacheEntries, CacheDownloadCommandSerializeMarketEntryError> {
        use CacheDownloadCommandSerializeMarketEntryError::*;
        let market_precise = handle!(
            MarketResponsePrecise::try_from(market.clone()),
            MarketResponseTryFromFailed,
            market: Box::new(market)
        );
        let market_round_trip = MarketResponse::from(market_precise.clone());
        handle_bool!(
            market != market_round_trip,
            RoundTripFailed,
            diff: format_debug_diff(&market, &market_round_trip, "market", "market_round_trip"),
            market: Box::new(market),
            market_round_trip: Box::new(market_round_trip)
        );
        let market_response_bytes = handle!(
            bitcode::serialize(&market),
            SerializeMarketResponseFailed,
            market: Box::new(market)
        );
        let market_entry_opt = match Market::maybe_try_from_market_response_precise(market_precise) {
            None => None,
            Some(result) => {
                let market = handle!(result, MarketTryFromFailed, market_slug);
                let market_key = market.slug.to_string();
                let market_bytes = handle!(
                    rkyv::to_bytes::<rkyv::rancor::Error>(&market),
                    SerializeMarketFailed,
                    market: Box::new(market)
                );
                Some((market_key, market_bytes.into_vec()))
            }
        };
        Ok(((market_slug, market_response_bytes), market_entry_opt))
    }

    fn serialize_orderbook_entry(orderbook: OrderBookSummaryResponse) -> Result<(TokenId, Vec<u8>), CacheDownloadCommandSerializeOrderbookEntryError> {
        use CacheDownloadCommandSerializeOrderbookEntryError::*;
        let orderbook = handle!(Self::round_trip_entry::<OrderBookSummaryResponse, OrderBookSummaryResponsePrecise, ConvertOrderBookSummaryResponseToOrderbookError>(orderbook), RoundTripEntryFailed);
        let bytes = handle!(
            bitcode::serialize(&orderbook),
            SerializeFailed,
            orderbook: Box::new(orderbook)
        );
        Ok((orderbook.asset_id, bytes))
    }

    fn serialize_event_entry((event_id, event): (String, Event)) -> Result<(String, Vec<u8>), CacheDownloadCommandSerializeEventEntryError> {
        use CacheDownloadCommandSerializeEventEntryError::*;
        let bytes = handle!(
            bitcode::serialize(&event),
            SerializeFailed,
            event: Box::new(event)
        );
        Ok((event_id, bytes))
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
    #[error("failed to parse '{len}' market responses", len = source.len())]
    MarketEntryFromResponseFailed { source: ErrVec<CacheDownloadCommandMarketEntryFromResponseError> },
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
pub enum CacheDownloadCommandMarketEntryFromResponseError {
    #[error("market response has empty market slug")]
    MarketSlugInvalid { market: Box<MarketResponse> },
    #[error("market response has duplicate market slug '{market_slug}'")]
    MarketSlugDuplicateInvalid { market_slug: String },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandEventEntryFromResponseError {
    #[error("event response has empty event id")]
    EventIdInvalid { event: Box<Event> },
    #[error("event response has missing event slug")]
    EventSlugMissingInvalid { event: Box<Event> },
    #[error("event response has empty event slug '{event_slug}'")]
    EventSlugInvalid { event_slug: String },
    #[error("event response has duplicate event slug '{event_slug}'")]
    EventSlugDuplicateInvalid { event_slug: String },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandFetchOrderbooksForTokensError {
    #[error("failed to fetch order books for '{len}' chunks", len = source.len())]
    FetchOrderbooksChunkFailed { source: ErrVec<CacheDownloadCommandFetchOrderbooksChunkError> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandFetchOrderbooksChunkError {
    #[error("failed to fetch order books for chunk")]
    OrderBooksFailed { source: polymarket_client_sdk::error::Error, requests: Box<[OrderBookSummaryRequest]> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandWritePageToDatabaseError {
    #[error("failed to serialize '{len}' market entries", len = source.len())]
    SerializeMarketEntryFailed { source: ErrVec<CacheDownloadCommandSerializeMarketEntryError> },
    #[error("failed to serialize '{len}' order book summaries", len = source.len())]
    SerializeOrderbookEntryFailed { source: ErrVec<CacheDownloadCommandSerializeOrderbookEntryError> },
    #[error("failed to insert '{len}' market response entries", len = source.len())]
    InsertMarketResponseEntriesFailed { source: ErrVec<CacheDownloadCommandInsertEntryError> },
    #[error("failed to insert '{len}' market entries", len = source.len())]
    InsertMarketEntriesFailed { source: ErrVec<CacheDownloadCommandInsertEntryError> },
    #[error("failed to insert '{len}' order book entries", len = source.len())]
    InsertOrderbookEntriesFailed { source: ErrVec<CacheDownloadCommandInsertEntryError> },
    #[error("failed to commit database transaction")]
    CommitTransactionFailed { source: fjall::Error },
    #[error("failed to persist database changes")]
    PersistDatabaseFailed { source: fjall::Error },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandWriteEventsToDatabaseError {
    #[error("failed to parse '{len}' event responses", len = source.len())]
    EventEntryFromResponseFailed { source: ErrVec<CacheDownloadCommandEventEntryFromResponseError> },
    #[error("failed to serialize '{len}' event responses", len = source.len())]
    SerializeEventEntryFailed { source: ErrVec<CacheDownloadCommandSerializeEventEntryError> },
    #[error("failed to insert '{len}' event entries", len = source.len())]
    InsertEventEntriesFailed { source: ErrVec<CacheDownloadCommandInsertEntryError> },
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
pub enum CacheDownloadCommandInsertEntryError {}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandInsertOrderbookEntryError {
    #[error("failed to check if order book key exists for token '{token_id}'")]
    CheckOrderbookKeyExistsFailed { source: fjall::Error, token_id: TokenId },
    #[error("order book key already exists for token '{token_id}'")]
    OrderbookKeyAlreadyExists { token_id: TokenId },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandSerializeMarketEntryError {
    #[error("failed to convert market response to precise type")]
    MarketResponseTryFromFailed { source: Box<ConvertMarketResponseToMarketError>, market: Box<MarketResponse> },
    #[error("round-tripped market response does not match original: '{diff}'")]
    RoundTripFailed { market: Box<MarketResponse>, market_round_trip: Box<MarketResponse>, diff: String },
    #[error("failed to serialize market response")]
    SerializeMarketResponseFailed { source: bitcode::Error, market: Box<MarketResponse> },
    #[error("failed to convert market response to market for slug '{market_slug}'")]
    MarketTryFromFailed { source: Box<MarketFallible>, market_slug: String },
    #[error("failed to serialize market")]
    SerializeMarketFailed { source: rkyv::rancor::Error, market: Box<Market> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandSerializeOrderbookEntryError {
    #[error("failed to round-trip order book summary response")]
    RoundTripEntryFailed { source: CacheDownloadCommandRoundTripEntryError<OrderBookSummaryResponse, ConvertOrderBookSummaryResponseToOrderbookError> },
    #[error("failed to serialize order book summary")]
    SerializeFailed { source: bitcode::Error, orderbook: Box<OrderBookSummaryResponse> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandSerializeEventEntryError {
    #[error("failed to serialize event response")]
    SerializeFailed { source: bitcode::Error, event: Box<Event> },
}
