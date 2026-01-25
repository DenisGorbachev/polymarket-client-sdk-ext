use crate::{NEXT_CURSOR_STOP, NextCursor, ShouldDownloadOrderbooks, TokenId, progress_report_line, to_fjall_key_from_token_id};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use errgonomic::{ErrVec, handle, handle_bool, handle_iter};
use fjall::{KeyspaceCreateOptions, PersistMode, Readable, SingleWriterTxDatabase, SingleWriterTxKeyspace};
use futures::future::join_all;
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest;
use polymarket_client_sdk::clob::types::response::{MarketResponse, OrderBookSummaryResponse};
use std::num::NonZeroU64;
use std::path::PathBuf;
use thiserror::Error;

const DEFAULT_DB_DIR: &str = ".cache/db";
const DEFAULT_MARKET_KEYSPACE: &str = "clob_market_responses";
const DEFAULT_ORDERBOOK_KEYSPACE: &str = "clob_order_book_summary_responses";
const CURSOR_KEYSPACE: &str = "cache_download_cursors";
const CURSOR_KEY: &str = "markets_next_cursor";
const ORDERBOOKS_CHUNK_SIZE: usize = 500;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheDownloadCommand {
    #[arg(long)]
    pub market_response_page_limit: Option<NonZeroU64>,
    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,
    #[arg(long, default_value = DEFAULT_MARKET_KEYSPACE)]
    pub market_response_keyspace: String,
    #[arg(long, default_value = DEFAULT_ORDERBOOK_KEYSPACE)]
    pub order_book_summary_response_keyspace: String,
}

impl CacheDownloadCommand {
    pub async fn run(self) -> Result<(), CacheDownloadCommandRunError> {
        use CacheDownloadCommandRunError::*;
        let Self {
            market_response_page_limit,
            dir,
            market_response_keyspace,
            order_book_summary_response_keyspace,
        } = self;
        let page_limit_total = market_response_page_limit.map(|limit| limit.get());
        let db = handle!(SingleWriterTxDatabase::builder(&dir).open(), OpenDatabaseFailed, dir);
        let market_keyspace = handle!(
            db.keyspace(&market_response_keyspace, KeyspaceCreateOptions::default),
            OpenMarketKeyspaceFailed,
            keyspace: market_response_keyspace
        );
        let orderbook_keyspace = handle!(
            db.keyspace(&order_book_summary_response_keyspace, KeyspaceCreateOptions::default),
            OpenOrderBookSummaryKeyspaceFailed,
            keyspace: order_book_summary_response_keyspace
        );
        let cursor_keyspace = handle!(
            db.keyspace(CURSOR_KEYSPACE, KeyspaceCreateOptions::default),
            OpenCursorKeyspaceFailed,
            keyspace: CURSOR_KEYSPACE.to_string()
        );
        let mut next_cursor = handle!(Self::resolve_start_cursor(&db, &cursor_keyspace, &market_keyspace), ResolveStartCursorFailed);
        let client = Client::default();
        let mut downloaded_pages: u64 = 0;

        loop {
            if Self::limit_reached(downloaded_pages, market_response_page_limit) {
                break;
            }
            eprintln!("{}", progress_report_line("Downloading markets pages", downloaded_pages.saturating_add(1), page_limit_total));
            let cursor_opt = Self::cursor_option(&next_cursor);
            let page = handle!(client.markets(cursor_opt).await, FetchMarketsFailed, next_cursor);
            downloaded_pages = downloaded_pages.saturating_add(1);
            let page_next_cursor = page.next_cursor.clone();
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
            let store_cursor = !Self::is_base64_offset_cursor(&page_next_cursor);
            handle!(Self::write_page_to_database(&db, &market_keyspace, &orderbook_keyspace, &cursor_keyspace, markets_to_store, orderbooks, page_next_cursor.clone(), store_cursor), WritePageToDatabaseFailed);
            if page_next_cursor == NEXT_CURSOR_STOP || Self::limit_reached(downloaded_pages, market_response_page_limit) {
                break;
            }
            next_cursor = page_next_cursor;
        }
        Ok(())
    }

    fn resolve_start_cursor(db: &SingleWriterTxDatabase, cursor_keyspace: &SingleWriterTxKeyspace, market_keyspace: &SingleWriterTxKeyspace) -> Result<NextCursor, CacheDownloadCommandResolveStartCursorError> {
        use CacheDownloadCommandResolveStartCursorError::*;
        let read_tx = db.read_tx();
        let cursor_bytes_opt = handle!(
            read_tx.get(cursor_keyspace, CURSOR_KEY),
            ReadCursorFailed,
            key: CURSOR_KEY.to_string()
        );
        match cursor_bytes_opt {
            Some(cursor_bytes) => {
                let cursor_vec = cursor_bytes.as_ref().to_vec();
                let cursor = handle!(String::from_utf8(cursor_vec), CursorValueInvalid);
                if Self::is_base64_offset_cursor(&cursor) {
                    let count = market_keyspace.approximate_len();
                    let count = handle!(u64::try_from(count), MarketCountConversionFailed, count);
                    Ok(Self::encode_offset_cursor(count))
                } else {
                    Ok(cursor)
                }
            }
            None => {
                let count = market_keyspace.approximate_len();
                let count = handle!(u64::try_from(count), MarketCountConversionFailed, count);
                Ok(Self::encode_offset_cursor(count))
            }
        }
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
    fn write_page_to_database(db: &SingleWriterTxDatabase, market_keyspace: &SingleWriterTxKeyspace, orderbook_keyspace: &SingleWriterTxKeyspace, cursor_keyspace: &SingleWriterTxKeyspace, markets: Vec<(String, MarketResponse)>, orderbooks: Vec<OrderBookSummaryResponse>, next_cursor: NextCursor, store_cursor: bool) -> Result<(), CacheDownloadCommandWritePageToDatabaseError> {
        use CacheDownloadCommandWritePageToDatabaseError::*;
        let serialized_markets = handle_iter!(markets.into_iter().map(Self::serialize_market_entry), SerializeMarketEntryFailed);
        let serialized_orderbooks = handle_iter!(orderbooks.into_iter().map(Self::serialize_orderbook_entry), SerializeOrderbookEntryFailed);
        let mut tx = db.write_tx();
        serialized_markets
            .into_iter()
            .for_each(|(market_slug, bytes)| {
                tx.insert(market_keyspace, market_slug.as_str(), bytes);
            });
        serialized_orderbooks
            .into_iter()
            .for_each(|(token_id, bytes)| {
                let key = to_fjall_key_from_token_id(token_id);
                tx.insert(orderbook_keyspace, key, bytes);
            });
        if store_cursor {
            tx.insert(cursor_keyspace, CURSOR_KEY, next_cursor.as_str());
        }
        handle!(tx.commit(), CommitTransactionFailed);
        handle!(db.persist(PersistMode::SyncAll), PersistDatabaseFailed);
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

    fn cursor_option(cursor: &str) -> Option<String> {
        if cursor.is_empty() { None } else { Some(cursor.to_string()) }
    }

    fn encode_offset_cursor(offset: u64) -> String {
        STANDARD.encode(offset.to_string())
    }

    fn decode_offset_cursor(cursor: &str) -> Option<i64> {
        let decoded = match STANDARD.decode(cursor) {
            Ok(decoded) => decoded,
            Err(_) => return None,
        };
        let decoded_str = match core::str::from_utf8(&decoded) {
            Ok(decoded_str) => decoded_str,
            Err(_) => return None,
        };
        decoded_str.parse::<i64>().ok()
    }

    fn is_base64_offset_cursor(cursor: &str) -> bool {
        Self::decode_offset_cursor(cursor).is_some()
    }

    fn limit_reached(downloaded: u64, limit: Option<NonZeroU64>) -> bool {
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
    #[error("failed to open cursor keyspace '{keyspace}'")]
    OpenCursorKeyspaceFailed { source: fjall::Error, keyspace: String },
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
    #[error("failed to read stored cursor at key '{key}'")]
    ReadCursorFailed { source: fjall::Error, key: String },
    #[error("stored cursor is not valid utf-8")]
    CursorValueInvalid { source: std::string::FromUtf8Error },
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
    #[error("failed to commit database transaction")]
    CommitTransactionFailed { source: fjall::Error },
    #[error("failed to persist database changes")]
    PersistDatabaseFailed { source: fjall::Error },
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
