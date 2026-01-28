use crate::{NEXT_CURSOR_STOP, NextCursor, ShouldDownloadOrderbooks, TokenId, progress_report_line};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use errgonomic::{ErrVec, handle, handle_bool, handle_iter};
use fjall::{KeyspaceCreateOptions, PersistMode, Readable, SingleWriterTxDatabase, SingleWriterTxKeyspace, SingleWriterWriteTx};
use futures::future::join_all;
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest;
use polymarket_client_sdk::clob::types::response::{MarketResponse, OrderBookSummaryResponse};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

pub const DEFAULT_DB_DIR: &str = ".cache/db";
pub const CLOB_MARKET_RESPONSE_KEYSPACE: &str = "clob_market_responses";
pub const CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE: &str = "clob_order_book_summary_responses";
const ORDERBOOKS_CHUNK_SIZE: usize = 500;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheDownloadCommand {
    #[arg(long)]
    pub market_response_page_limit: Option<NonZeroUsize>,
    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,
}

impl CacheDownloadCommand {
    pub async fn run(self) -> Result<ExitCode, CacheDownloadCommandRunError> {
        use CacheDownloadCommandRunError::*;
        let Self {
            market_response_page_limit,
            dir,
        } = self;
        let page_limit_total = market_response_page_limit.map(|limit| limit.get());
        let db = handle!(SingleWriterTxDatabase::builder(&dir).open(), OpenDatabaseFailed, dir);
        let market_keyspace = handle!(
            db.keyspace(CLOB_MARKET_RESPONSE_KEYSPACE, KeyspaceCreateOptions::default),
            OpenMarketKeyspaceFailed,
            keyspace: CLOB_MARKET_RESPONSE_KEYSPACE.to_string()
        );
        let orderbook_keyspace = handle!(
            db.keyspace(CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE, KeyspaceCreateOptions::default),
            OpenOrderBookSummaryKeyspaceFailed,
            keyspace: CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE.to_string()
        );
        let next_cursor = handle!(Self::resolve_start_cursor(&market_keyspace), ResolveStartCursorFailed);
        let client = Client::default();
        let mut downloaded_pages: usize = 0;

        loop {
            if Self::limit_reached(downloaded_pages, market_response_page_limit) {
                break;
            }
            eprintln!("{}", progress_report_line("Downloading markets pages", downloaded_pages.saturating_add(1), None, page_limit_total));
            let page = handle!(client.markets(Some(next_cursor.clone())).await, FetchMarketsFailed, next_cursor);
            downloaded_pages = downloaded_pages.saturating_add(1);
            let next_cursor = page.next_cursor;
            let markets = page.data;
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
            let orderbooks = handle!(Self::fetch_orderbooks_for_tokens(&client, &token_ids).await, FetchOrderbooksForTokensFailed);
            handle!(Self::write_page_to_database(&db, &market_keyspace, &orderbook_keyspace, markets_to_store, orderbooks), WritePageToDatabaseFailed);
            if next_cursor == NEXT_CURSOR_STOP || Self::limit_reached(downloaded_pages, market_response_page_limit) {
                break;
            }
        }
        Ok(ExitCode::SUCCESS)
    }

    fn resolve_start_cursor(market_keyspace: &SingleWriterTxKeyspace) -> Result<NextCursor, CacheDownloadCommandResolveStartCursorError> {
        use CacheDownloadCommandResolveStartCursorError::*;
        let count = handle!(market_keyspace.as_ref().len(), LenFailed);
        let count = handle!(u64::try_from(count), MarketCountConversionFailed, count);
        Ok(STANDARD.encode(count.to_string()))
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

    async fn fetch_orderbooks_for_tokens(client: &Client, token_ids: &[TokenId]) -> Result<Vec<OrderBookSummaryResponse>, CacheDownloadCommandFetchOrderbooksForTokensError> {
        use CacheDownloadCommandFetchOrderbooksForTokensError::*;
        let futures = token_ids
            .chunks(ORDERBOOKS_CHUNK_SIZE)
            .map(|chunk| Self::fetch_orderbooks_chunk(client, chunk));
        let results = join_all(futures).await;
        let orderbooks = handle_iter!(results.into_iter(), FetchOrderbooksChunkFailed);
        Ok(orderbooks.into_iter().flatten().collect())
    }

    async fn fetch_orderbooks_chunk(client: &Client, token_ids: &[TokenId]) -> Result<Vec<OrderBookSummaryResponse>, CacheDownloadCommandFetchOrderbooksChunkError> {
        use CacheDownloadCommandFetchOrderbooksChunkError::*;
        let requests = token_ids
            .iter()
            .copied()
            .map(Self::build_orderbook_request)
            .collect::<Vec<_>>();
        let orderbooks = handle!(client.order_books(&requests).await, OrderBooksFailed, requests: requests.into_boxed_slice());
        Ok(orderbooks)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_page_to_database(db: &SingleWriterTxDatabase, market_keyspace: &SingleWriterTxKeyspace, orderbook_keyspace: &SingleWriterTxKeyspace, markets: Vec<(String, MarketResponse)>, orderbooks: Vec<OrderBookSummaryResponse>) -> Result<(), CacheDownloadCommandWritePageToDatabaseError> {
        use CacheDownloadCommandWritePageToDatabaseError::*;
        let serialized_markets = handle_iter!(markets.into_iter().map(Self::serialize_market_entry), SerializeMarketEntryFailed);
        let serialized_orderbooks = handle_iter!(orderbooks.into_iter().map(Self::serialize_orderbook_entry), SerializeOrderbookEntryFailed);
        let mut tx = db.write_tx();
        let _market_inserts = handle_iter!(
            serialized_markets
                .into_iter()
                .map(|(market_slug, bytes)| { Self::insert_entry(&mut tx, market_keyspace, market_slug, bytes) }),
            InsertMarketEntriesFailed
        );
        let _orderbook_inserts = handle_iter!(
            serialized_orderbooks
                .into_iter()
                .map(|(token_id, bytes)| { Self::insert_entry(&mut tx, orderbook_keyspace, token_id.to_string(), bytes) }),
            InsertOrderbookEntriesFailed
        );
        handle!(tx.commit(), CommitTransactionFailed);
        handle!(db.persist(PersistMode::SyncAll), PersistDatabaseFailed);
        Ok(())
    }

    fn insert_entry(tx: &mut SingleWriterWriteTx, keyspace: &SingleWriterTxKeyspace, key: String, value: Vec<u8>) -> Result<(), CacheDownloadCommandInsertEntryError> {
        use CacheDownloadCommandInsertEntryError::*;
        let exists = handle!(tx.contains_key(keyspace, &key), ContainsKeyFailed, key, value);
        handle_bool!(exists, KeyAlreadyExists, key, value);
        tx.insert(keyspace, key, value);
        Ok(())
    }

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

    fn build_orderbook_request(token_id: TokenId) -> OrderBookSummaryRequest {
        OrderBookSummaryRequest::builder()
            .token_id(token_id)
            .build()
    }

    fn limit_reached(downloaded: usize, limit: Option<NonZeroUsize>) -> bool {
        match limit {
            Some(limit) => downloaded >= limit.get(),
            None => false,
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandRunError {
    #[error("failed to open database at '{dir}'")]
    OpenDatabaseFailed { source: fjall::Error, dir: PathBuf },
    #[error("failed to open market keyspace '{keyspace}'")]
    OpenMarketKeyspaceFailed { source: fjall::Error, keyspace: String },
    #[error("failed to open order book summary keyspace '{keyspace}'")]
    OpenOrderBookSummaryKeyspaceFailed { source: fjall::Error, keyspace: String },
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
pub enum CacheDownloadCommandResolveStartCursorError {
    #[error("failed to read market keyspace length")]
    LenFailed { source: fjall::Error },
    #[error("failed to convert market count '{count}' to cursor offset")]
    MarketCountConversionFailed { source: core::num::TryFromIntError, count: usize },
}

#[derive(Error, Debug)]
pub enum CacheDownloadCommandMarketEntryFromResponseError {
    #[error("market response has empty market slug")]
    MarketSlugInvalid { market: Box<MarketResponse> },
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
