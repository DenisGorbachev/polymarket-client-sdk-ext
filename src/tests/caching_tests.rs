use crate::{MARKET_RESPONSE_PAGE_CACHE_LIMIT_ENV, NEXT_CURSOR_STOP, ParseEnvVarError, ShouldDownloadOrderbooks, TokenId, cache_dir_path, market_response_cache_path, orderbook_summary_response_cache_path, parse_env_var, progress_report_line, to_tmp_path};
use errgonomic::{exit_result, handle};
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest;
use polymarket_client_sdk::clob::types::response::{MarketResponse, OrderBookSummaryResponse};
use polymarket_client_sdk::error::{Status, StatusCode};
use serde::Serialize;
use std::io;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

#[tokio::test]
pub async fn must_refresh_cache() -> ExitCode {
    exit_result(refresh_caches().await)
}

#[allow(clippy::question_mark)]
pub async fn refresh_caches() -> Result<(), RefreshCachesError> {
    use RefreshCachesError::*;
    let page_limit = handle!(parse_env_var(MARKET_RESPONSE_PAGE_CACHE_LIMIT_ENV, parse_page_limit), MarketResponsePageCacheLimitFailed);
    let page_limit_total = page_limit.map(|limit| limit.get());
    let market_cache_path = market_response_cache_path();
    let orderbook_cache_path = orderbook_summary_response_cache_path();
    let market_temp_path = to_tmp_path(&market_cache_path);
    let orderbook_temp_path = to_tmp_path(&orderbook_cache_path);
    let cache_dir = cache_dir_path();
    handle!(fs::create_dir_all(&cache_dir).await, CreateCacheDirFailed, cache_dir);
    let mut market_file = handle!(File::create(&market_temp_path).await, CreateTempFileFailed, temp_path: market_temp_path);
    let mut orderbook_file = handle!(File::create(&orderbook_temp_path).await, CreateTempFileFailed, temp_path: orderbook_temp_path);
    let client = Client::default();
    let mut downloaded_markets: u64 = 0;
    let mut downloaded_orderbooks: u64 = 0;
    let mut downloaded_pages: u64 = 0;
    let mut orderbook_requests: u64 = 0;
    let mut next_cursor: Option<String> = None;

    loop {
        if limit_reached(downloaded_pages, page_limit) {
            break;
        }
        eprintln!("{}", progress_report_line("Requesting markets pages", downloaded_pages.saturating_add(1), page_limit_total));
        let page = handle!(client.markets(next_cursor.clone()).await, FetchMarketsFailed, next_cursor);
        downloaded_pages = downloaded_pages.saturating_add(1);
        let page_next_cursor = page.next_cursor.clone();
        let markets = page.data;
        if markets.is_empty() {
            break;
        }
        let mut token_ids = Vec::new();
        for market in markets {
            if market.should_download_orderbooks() {
                token_ids.extend(market.tokens.iter().map(|token| token.token_id));
            }
            let write_result = write_jsonl_record(
                &mut market_file,
                &market_temp_path,
                market,
                |source, market| SerializeMarketFailed {
                    source,
                    market: Box::new(market),
                },
                |source, temp_path| WriteMarketTempFileFailed {
                    source,
                    temp_path,
                },
            )
            .await;
            if let Err(error) = write_result {
                return Err(error);
            }
            downloaded_markets = downloaded_markets.saturating_add(1);
            if downloaded_markets.is_multiple_of(100) {
                eprintln!("{}", progress_report_line("Downloading markets", downloaded_markets, None));
            }
        }
        let fetch_result = fetch_orderbooks_for_tokens(&client, &token_ids, &mut orderbook_file, &orderbook_temp_path, &mut downloaded_orderbooks, &mut orderbook_requests).await;
        if let Err(error) = fetch_result {
            return Err(error);
        }
        if page_next_cursor == NEXT_CURSOR_STOP || limit_reached(downloaded_pages, page_limit) {
            break;
        }
        next_cursor = Some(page_next_cursor);
    }
    if !downloaded_markets.is_multiple_of(100) {
        eprintln!("{}", progress_report_line("Downloading markets", downloaded_markets, None));
    }
    if !downloaded_orderbooks.is_multiple_of(100) {
        eprintln!("{}", progress_report_line("Downloading orderbooks", downloaded_orderbooks, None));
    }
    handle!(market_file.flush().await, FlushTempFileFailed, temp_path: market_temp_path);
    handle!(market_file.sync_all().await, SyncTempFileFailed, temp_path: market_temp_path);
    handle!(orderbook_file.flush().await, FlushTempFileFailed, temp_path: orderbook_temp_path);
    handle!(orderbook_file.sync_all().await, SyncTempFileFailed, temp_path: orderbook_temp_path);
    drop(market_file);
    drop(orderbook_file);
    handle!(fs::rename(&market_temp_path, &market_cache_path).await, PersistTempFileFailed, temp_path: market_temp_path, cache_path: market_cache_path);
    handle!(fs::rename(&orderbook_temp_path, &orderbook_cache_path).await, PersistTempFileFailed, temp_path: orderbook_temp_path, cache_path: orderbook_cache_path);
    Ok(())
}

async fn append_jsonl_line(file: &mut File, line: &str) -> Result<(), io::Error> {
    let write_result = file.write_all(line.as_bytes()).await;
    match write_result {
        Ok(()) => {
            let newline_result = file.write_all(b"\n").await;
            match newline_result {
                Ok(()) => Ok(()),
                Err(source) => Err(source),
            }
        }
        Err(source) => Err(source),
    }
}

async fn write_jsonl_record<T>(file: &mut File, temp_path: &Path, value: T, serialize_error: impl FnOnce(serde_json::Error, T) -> RefreshCachesError, write_error: impl FnOnce(io::Error, PathBuf) -> RefreshCachesError) -> Result<(), RefreshCachesError>
where
    T: Serialize,
{
    let line = match serde_json::to_string(&value) {
        Ok(line) => line,
        Err(source) => {
            return Err(serialize_error(source, value));
        }
    };
    let write_result = append_jsonl_line(file, &line).await;
    match write_result {
        Ok(()) => Ok(()),
        Err(source) => Err(write_error(source, temp_path.to_path_buf())),
    }
}

#[allow(clippy::question_mark)]
async fn fetch_orderbooks_for_tokens(client: &Client, token_ids: &[TokenId], orderbook_file: &mut File, orderbook_temp_path: &Path, downloaded_orderbooks: &mut u64, orderbook_requests: &mut u64) -> Result<(), RefreshCachesError> {
    use RefreshCachesError::*;
    let mut ranges = Vec::new();
    ranges.push((0usize, token_ids.len()));
    while let Some((start, end)) = ranges.pop() {
        let slice = &token_ids[start..end];
        if slice.is_empty() {
            continue;
        }
        let requests = slice
            .iter()
            .map(|token_id| {
                OrderBookSummaryRequest::builder()
                    .token_id(*token_id)
                    .build()
            })
            .collect::<Vec<_>>();
        *orderbook_requests = orderbook_requests.saturating_add(1);
        eprintln!("{}", progress_report_line("Requesting orderbooks", *orderbook_requests, None));
        let orderbooks_result = client.order_books(&requests).await;
        let orderbooks = match orderbooks_result {
            Ok(orderbooks) => orderbooks,
            Err(source) => {
                if is_payload_limit_error(&source) && slice.len() > 1 {
                    let split_at = start + (slice.len() / 2);
                    ranges.push((split_at, end));
                    ranges.push((start, split_at));
                    continue;
                }
                return Err(FetchOrderBooksFailed {
                    source,
                    requests: requests.into_boxed_slice(),
                });
            }
        };
        for orderbook in orderbooks {
            let write_result = write_jsonl_record(
                orderbook_file,
                orderbook_temp_path,
                orderbook,
                |source, orderbook| SerializeOrderBookFailed {
                    source,
                    orderbook: Box::new(orderbook),
                },
                |source, temp_path| WriteOrderBookTempFileFailed {
                    source,
                    temp_path,
                },
            )
            .await;
            if let Err(error) = write_result {
                return Err(error);
            }
            *downloaded_orderbooks = downloaded_orderbooks.saturating_add(1);
            if downloaded_orderbooks.is_multiple_of(100) {
                eprintln!("{}", progress_report_line("Downloading orderbooks", *downloaded_orderbooks, None));
            }
        }
    }
    Ok(())
}

fn is_payload_limit_error(error: &polymarket_client_sdk::error::Error) -> bool {
    let Some(status) = error.downcast_ref::<Status>() else {
        return false;
    };
    status.status_code == StatusCode::BAD_REQUEST && status.message.contains("Payload exceeds the limit")
}

fn parse_page_limit(value: String) -> Result<NonZeroU64, ParsePageLimitError> {
    use ParsePageLimitError::*;
    let parsed = match value.parse::<u64>() {
        Ok(parsed) => parsed,
        Err(source) => {
            return Err(ParseValueFailed {
                source,
                value,
            });
        }
    };
    let limit = match NonZeroU64::new(parsed) {
        Some(limit) => limit,
        None => {
            return Err(ZeroValueInvalid {
                value,
            });
        }
    };
    Ok(limit)
}

fn limit_reached(downloaded: u64, limit: Option<NonZeroU64>) -> bool {
    match limit {
        Some(limit) => downloaded >= limit.get(),
        None => false,
    }
}

#[derive(Error, Debug)]
pub enum RefreshCachesError {
    #[error("failed to read market response page cache limit env var")]
    MarketResponsePageCacheLimitFailed { source: ParseEnvVarError<ParsePageLimitError> },
    #[error("failed to create cache directory '{cache_dir}'")]
    CreateCacheDirFailed { source: io::Error, cache_dir: PathBuf },
    #[error("failed to create temp cache file '{temp_path}'")]
    CreateTempFileFailed { source: io::Error, temp_path: PathBuf },
    #[error("failed to fetch markets page")]
    FetchMarketsFailed { source: polymarket_client_sdk::error::Error, next_cursor: Option<String> },
    #[error("failed to serialize market response")]
    SerializeMarketFailed { source: serde_json::Error, market: Box<MarketResponse> },
    #[error("failed to write market cache temp file '{temp_path}'")]
    WriteMarketTempFileFailed { source: io::Error, temp_path: PathBuf },
    #[error("failed to fetch orderbooks")]
    FetchOrderBooksFailed { source: polymarket_client_sdk::error::Error, requests: Box<[OrderBookSummaryRequest]> },
    #[error("failed to serialize orderbook response")]
    SerializeOrderBookFailed { source: serde_json::Error, orderbook: Box<OrderBookSummaryResponse> },
    #[error("failed to write orderbook cache temp file '{temp_path}'")]
    WriteOrderBookTempFileFailed { source: io::Error, temp_path: PathBuf },
    #[error("failed to flush temp cache file '{temp_path}'")]
    FlushTempFileFailed { source: io::Error, temp_path: PathBuf },
    #[error("failed to sync temp cache file '{temp_path}'")]
    SyncTempFileFailed { source: io::Error, temp_path: PathBuf },
    #[error("failed to persist temp cache file '{temp_path}' to '{cache_path}'")]
    PersistTempFileFailed { source: io::Error, temp_path: PathBuf, cache_path: PathBuf },
}

#[derive(Error, Debug)]
pub enum ParsePageLimitError {
    #[error("failed to parse page limit value '{value}'")]
    ParseValueFailed { source: core::num::ParseIntError, value: String },
    #[error("page limit value '{value}' must be non-zero")]
    ZeroValueInvalid { value: String },
}
