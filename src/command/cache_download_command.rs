use crate::{CLOB_MARKET_RESPONSES_KEYSPACE, CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE, GAMMA_EVENTS_KEYSPACE, NEXT_CURSOR_STOP, NextCursor, ShouldDownloadOrderbooks, TokenId, progress_report_line};
use crate::{DEFAULT_DB_DIR, GAMMA_EVENTS_PAGE_SIZE};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use errgonomic::{ErrVec, handle, handle_bool, handle_iter, map_err};
use fjall::{KeyspaceCreateOptions, PersistMode, SingleWriterTxDatabase, SingleWriterTxKeyspace};
use futures::future::join_all;
use polymarket_client_sdk::clob::Client as ClobClient;
use polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest;
use polymarket_client_sdk::clob::types::response::{MarketResponse, OrderBookSummaryResponse};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::EventsRequest;
use polymarket_client_sdk::gamma::types::response::Event;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;
use stub_macro::stub;
use thiserror::Error;

const ORDERBOOKS_CHUNK_SIZE: usize = 500;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheDownloadCommand {
    /// A limit on the number of downloaded pages (applies to all paginated endpoints)
    #[arg(long)]
    pub page_limit: Option<NonZeroUsize>,

    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,
}

impl CacheDownloadCommand {
    pub async fn run(self) -> Result<ExitCode, CacheDownloadCommandRunError> {
        use CacheDownloadCommandRunError::*;
        let Self {
            page_limit,
            dir,
        } = self;
        let db = handle!(SingleWriterTxDatabase::builder(&dir).open(), OpenDatabaseFailed, dir);
        let market_keyspace = handle!(
            db.keyspace(CLOB_MARKET_RESPONSES_KEYSPACE, KeyspaceCreateOptions::default),
            KeyspaceOpenFailed,
            keyspace: CLOB_MARKET_RESPONSES_KEYSPACE
        );
        let orderbook_keyspace = handle!(
            db.keyspace(CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE, KeyspaceCreateOptions::default),
            KeyspaceOpenFailed,
            keyspace: CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE
        );
        let event_keyspace = handle!(
            db.keyspace(GAMMA_EVENTS_KEYSPACE, KeyspaceCreateOptions::default),
            KeyspaceOpenFailed,
            keyspace: GAMMA_EVENTS_KEYSPACE
        );
        let clob_client = ClobClient::default();
        let gamma_client = GammaClient::default();
        let page_limit = page_limit.map(NonZeroUsize::get);
        let markets_download = async {
            use CacheDownloadCommandRunError::*;
            map_err!(Self::download_market_responses(&db, &market_keyspace, &orderbook_keyspace, &clob_client, page_limit).await, DownloadMarketResponsesFailed)
        };
        let events_download = async {
            use CacheDownloadCommandRunError::*;
            map_err!(Self::download_gamma_events(&db, &event_keyspace, &gamma_client, page_limit).await, DownloadGammaEventsFailed)
        };
        let result = tokio::try_join!(markets_download, events_download);
        match result {
            Ok((_markets, _events)) => (),
            Err(error) => return Err(error),
        }
        Ok(ExitCode::SUCCESS)
    }

    async fn download_market_responses(db: &SingleWriterTxDatabase, market_keyspace: &SingleWriterTxKeyspace, orderbook_keyspace: &SingleWriterTxKeyspace, client: &ClobClient, page_limit: Option<usize>) -> Result<(), CacheDownloadCommandDownloadMarketResponsesError> {
        use CacheDownloadCommandDownloadMarketResponsesError::*;
        let offset = stub!(usize, "Use handle!(market_keyspace.as_ref().len(), LenFailed)");
        let mut next_cursor = handle!(Self::resolve_start_offset(market_keyspace), ResolveStartCursorFailed);
        let mut page_offset: usize = 0;

        loop {
            eprintln!("{}", progress_report_line("Downloading markets pages", offset, None, None, page_offset, page_limit));
            let page = handle!(client.markets(Some(next_cursor.clone())).await, FetchMarketsFailed, next_cursor);
            page_offset = page_offset.saturating_add(1);
            let markets = page.data;
            let next_cursor_new = page.next_cursor;
            if markets.is_empty() {
                break;
            }
            let market_entries = handle_iter!(markets.into_iter().map(Self::market_entry_from_response), MarketEntryFromResponseFailed);
            let token_ids = market_entries
                .iter()
                .flat_map(|(_, _, token_ids)| token_ids.iter().copied())
                .collect::<Vec<_>>();
            let markets_to_store = market_entries
                .into_iter()
                .map(|(market_slug, market, _)| (market_slug, market))
                .collect::<Vec<_>>();
            let orderbooks = handle!(Self::fetch_orderbooks_for_tokens(client, &token_ids).await, FetchOrderbooksForTokensFailed);
            handle!(Self::write_market_response_page_to_database(db, market_keyspace, orderbook_keyspace, markets_to_store, orderbooks), WritePageToDatabaseFailed);
            next_cursor = next_cursor_new;
            if next_cursor == NEXT_CURSOR_STOP || Self::limit_reached(page_offset, page_limit) {
                break;
            }
        }
        Ok(())
    }

    async fn download_gamma_events(db: &SingleWriterTxDatabase, event_keyspace: &SingleWriterTxKeyspace, client: &GammaClient, page_limit: Option<usize>) -> Result<(), CacheDownloadCommandDownloadGammaEventsError> {
        use CacheDownloadCommandDownloadGammaEventsError::*;
        let offset = stub!(usize, "Use handle!(event_keyspace.as_ref().len(), LenFailed)");
        let mut offset_i32 = handle!(Self::resolve_start_event_offset(event_keyspace), ResolveStartEventOffsetFailed);
        let mut page_offset = 0;

        loop {
            eprintln!("{}", progress_report_line("Downloading events pages", offset, None, None, page_offset, page_limit));
            let request = EventsRequest::builder()
                .order(vec!["id".to_string()])
                .ascending(true)
                .limit(GAMMA_EVENTS_PAGE_SIZE)
                .offset(offset_i32)
                .build();
            let events = handle!(client.events(&request).await, FetchEventsFailed, request: Box::new(request));
            if events.is_empty() {
                break;
            }
            let event_count = events.len();
            handle!(Self::write_events_to_database(db, event_keyspace, events), WriteEventsToDatabaseFailed);
            let event_count = handle!(i32::try_from(event_count), EventCountConversionFailed, count: event_count);
            offset_i32 = offset_i32.saturating_add(event_count);
            page_offset = page_offset.saturating_add(1);
            if event_count < GAMMA_EVENTS_PAGE_SIZE || Self::limit_reached(page_offset, page_limit) {
                break;
            }
        }
        Ok(())
    }

    // TODO: Remove this function
    fn resolve_start_offset(market_keyspace: &SingleWriterTxKeyspace) -> Result<NextCursor, CacheDownloadCommandResolveStartCursorError> {
        use CacheDownloadCommandResolveStartCursorError::*;
        let count = handle!(market_keyspace.as_ref().len(), LenFailed);
        let count = handle!(u64::try_from(count), MarketCountConversionFailed, count);
        Ok(STANDARD.encode(count.to_string()))
    }

    // TODO: Remove this function
    fn resolve_start_event_offset(event_keyspace: &SingleWriterTxKeyspace) -> Result<i32, CacheDownloadCommandResolveStartEventOffsetError> {
        use CacheDownloadCommandResolveStartEventOffsetError::*;
        let count = handle!(event_keyspace.as_ref().len(), LenFailed);
        let offset = handle!(i32::try_from(count), EventCountConversionFailed, count);
        Ok(offset)
    }

    fn market_entry_from_response(market: MarketResponse) -> Result<(String, MarketResponse, Vec<TokenId>), CacheDownloadCommandMarketEntryFromResponseError> {
        use CacheDownloadCommandMarketEntryFromResponseError::*;
        let market_slug = market.market_slug.clone();
        handle_bool!(market_slug.trim().is_empty(), MarketSlugInvalid, market: Box::new(market));
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

    fn event_entry_from_response(event: Event) -> Result<(String, Event), CacheDownloadCommandEventEntryFromResponseError> {
        use CacheDownloadCommandEventEntryFromResponseError::*;
        let event_id = event.id.clone();
        handle_bool!(event_id.trim().is_empty(), EventIdInvalid, event: Box::new(event));
        Ok((event_id, event))
    }

    async fn fetch_orderbooks_for_tokens(client: &ClobClient, token_ids: &[TokenId]) -> Result<Vec<OrderBookSummaryResponse>, CacheDownloadCommandFetchOrderbooksForTokensError> {
        use CacheDownloadCommandFetchOrderbooksForTokensError::*;
        let futures = token_ids
            .chunks(ORDERBOOKS_CHUNK_SIZE)
            .map(|chunk| Self::fetch_orderbooks_chunk(client, chunk.iter()));
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
    fn write_market_response_page_to_database(db: &SingleWriterTxDatabase, market_keyspace: &SingleWriterTxKeyspace, orderbook_keyspace: &SingleWriterTxKeyspace, markets: Vec<(String, MarketResponse)>, orderbooks: Vec<OrderBookSummaryResponse>) -> Result<(), CacheDownloadCommandWritePageToDatabaseError> {
        use CacheDownloadCommandWritePageToDatabaseError::*;
        let serialized_markets = handle_iter!(markets.into_iter().map(Self::serialize_market_entry), SerializeMarketEntryFailed);
        let serialized_orderbooks = handle_iter!(orderbooks.into_iter().map(Self::serialize_orderbook_entry), SerializeOrderbookEntryFailed);
        let mut tx = db.write_tx();
        let _market_inserts = handle_iter!(
            serialized_markets.into_iter().map(|(market_slug, bytes)| {
                tx.insert(market_keyspace, market_slug, bytes);
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

    fn write_events_to_database(db: &SingleWriterTxDatabase, event_keyspace: &SingleWriterTxKeyspace, events: Vec<Event>) -> Result<(), CacheDownloadCommandWriteEventsToDatabaseError> {
        use CacheDownloadCommandWriteEventsToDatabaseError::*;
        let event_entries = handle_iter!(events.into_iter().map(Self::event_entry_from_response), EventEntryFromResponseFailed);
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

    // fn insert_entry(tx: &mut SingleWriterWriteTx, keyspace: &SingleWriterTxKeyspace, key: String, value: Vec<u8>) -> Result<(), CacheDownloadCommandInsertEntryError> {
    //     use CacheDownloadCommandInsertEntryError::*;
    //     let exists = handle!(tx.contains_key(keyspace, &key), ContainsKeyFailed, key, value);
    //     handle_bool!(exists, KeyAlreadyExists, key, value);
    //     tx.insert(keyspace, key, value);
    //     Ok(())
    // }

    fn serialize_market_entry((market_slug, market): (String, MarketResponse)) -> Result<(String, Vec<u8>), CacheDownloadCommandSerializeMarketEntryError> {
        use CacheDownloadCommandSerializeMarketEntryError::*;
        let bytes = handle!(
            serde_json::to_vec(&market),
            SerializeFailed,
            market: Box::new(market)
        );
        Ok((market_slug, bytes))
    }

    fn serialize_orderbook_entry(orderbook: OrderBookSummaryResponse) -> Result<(TokenId, Vec<u8>), CacheDownloadCommandSerializeOrderbookEntryError> {
        use CacheDownloadCommandSerializeOrderbookEntryError::*;
        let bytes = handle!(
            serde_json::to_vec(&orderbook),
            SerializeFailed,
            orderbook: Box::new(orderbook)
        );
        Ok((orderbook.asset_id, bytes))
    }

    fn serialize_event_entry((event_id, event): (String, Event)) -> Result<(String, Vec<u8>), CacheDownloadCommandSerializeEventEntryError> {
        use CacheDownloadCommandSerializeEventEntryError::*;
        let bytes = handle!(
            serde_json::to_vec(&event),
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
    #[error("failed to open keyspace '{keyspace}'")]
    KeyspaceOpenFailed { source: fjall::Error, keyspace: &'static str },
    #[error("failed to download market responses")]
    DownloadMarketResponsesFailed { source: CacheDownloadCommandDownloadMarketResponsesError },
    #[error("failed to download gamma events")]
    DownloadGammaEventsFailed { source: CacheDownloadCommandDownloadGammaEventsError },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandDownloadMarketResponsesError {
    #[error("failed to resolve start cursor")]
    ResolveStartCursorFailed { source: CacheDownloadCommandResolveStartCursorError },
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
    #[error("failed to resolve start event offset")]
    ResolveStartEventOffsetFailed { source: CacheDownloadCommandResolveStartEventOffsetError },
    #[error("failed to fetch gamma events page")]
    FetchEventsFailed { source: polymarket_client_sdk::error::Error, request: Box<EventsRequest> },
    #[error("failed to convert event count '{count}' to offset")]
    EventCountConversionFailed { source: core::num::TryFromIntError, count: usize },
    #[error("failed to persist events to database")]
    WriteEventsToDatabaseFailed { source: CacheDownloadCommandWriteEventsToDatabaseError },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandResolveStartCursorError {
    #[error("failed to read market keyspace length")]
    LenFailed { source: fjall::Error },
    #[error("failed to convert market count '{count}' to cursor offset")]
    MarketCountConversionFailed { source: core::num::TryFromIntError, count: usize },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandResolveStartEventOffsetError {
    #[error("failed to read event keyspace length")]
    LenFailed { source: fjall::Error },
    #[error("failed to convert event count '{count}' to start offset")]
    EventCountConversionFailed { source: core::num::TryFromIntError, count: usize },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandMarketEntryFromResponseError {
    #[error("market response has empty market slug")]
    MarketSlugInvalid { market: Box<MarketResponse> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandEventEntryFromResponseError {
    #[error("event response has empty event id")]
    EventIdInvalid { event: Box<Event> },
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
    #[error("failed to serialize '{len}' market responses", len = source.len())]
    SerializeMarketEntryFailed { source: ErrVec<CacheDownloadCommandSerializeMarketEntryError> },
    #[error("failed to serialize '{len}' order book summaries", len = source.len())]
    SerializeOrderbookEntryFailed { source: ErrVec<CacheDownloadCommandSerializeOrderbookEntryError> },
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
pub enum CacheDownloadCommandInsertEntryError {
    #[error("failed to call contains_key for '{key}'")]
    ContainsKeyFailed { source: fjall::Error, key: String, value: Vec<u8> },
    #[error("key already exists: '{key}'")]
    KeyAlreadyExists { key: String, value: Vec<u8> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandInsertOrderbookEntryError {
    #[error("failed to check if order book key exists for token '{token_id}'")]
    CheckOrderbookKeyExistsFailed { source: fjall::Error, token_id: TokenId },
    #[error("order book key already exists for token '{token_id}'")]
    OrderbookKeyAlreadyExists { token_id: TokenId },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandSerializeMarketEntryError {
    #[error("failed to serialize market response")]
    SerializeFailed { source: serde_json::Error, market: Box<MarketResponse> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandSerializeOrderbookEntryError {
    #[error("failed to serialize order book summary")]
    SerializeFailed { source: serde_json::Error, orderbook: Box<OrderBookSummaryResponse> },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandSerializeEventEntryError {
    #[error("failed to serialize event response")]
    SerializeFailed { source: serde_json::Error, event: Box<Event> },
}
