use async_jsonl::{Jsonl, JsonlDeserialize};
use core::error::Error as StdError;
use errgonomic::{handle, handle_bool};
use futures::{Stream, StreamExt};
use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const MARKET_RESPONSE_PAGE_CACHE_LIMIT_ENV: &str = "MARKET_RESPONSE_PAGE_CACHE_LIMIT";
pub const CACHE_DIR: &str = ".cache";
pub const MARKET_RESPONSE_CACHE_PATH: &str = "market_response.all.jsonl";
pub const ORDERBOOK_SUMMARY_RESPONSE_CACHE_PATH: &str = "orderbook_summary_response.all.jsonl";

pub fn parse_env_var<T, E>(var: &str, parse: impl FnOnce(String) -> Result<T, E>) -> Result<Option<T>, ParseEnvVarError<E>>
where
    E: StdError,
{
    use ParseEnvVarError::*;
    let value_opt_result = match env::var(var) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(source) => Err(source),
    };
    let value_opt = handle!(value_opt_result, ReadEnvVarFailed, var: var.to_string());
    let Some(value) = value_opt else {
        return Ok(None);
    };
    let value_for_parse = value.clone();
    let parsed = handle!(
        parse(value_for_parse),
        ParseValueFailed,
        var: var.to_string(),
        value
    );
    Ok(Some(parsed))
}

pub fn parse_boolish(value: &str) -> Result<bool, ParseBoolishError> {
    use ParseBoolishError::*;
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "1" | "true" | "t" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "f" | "no" | "n" | "off" => Ok(false),
        _ => Err(InvalidValue {
            value: value.to_string(),
        }),
    }
}

pub fn progress_report_line(action: &str, count: u64, total: Option<u64>) -> String {
    let counter = match total {
        None => format!("{count} so far"),
        Some(total) => format!("{count} / {total}"),
    };
    format!("{action}: {counter}")
}

pub fn cache_dir_path() -> PathBuf {
    PathBuf::from(CACHE_DIR)
}

pub fn market_response_cache_path() -> PathBuf {
    cache_dir_path().join(MARKET_RESPONSE_CACHE_PATH)
}

pub fn orderbook_summary_response_cache_path() -> PathBuf {
    cache_dir_path().join(ORDERBOOK_SUMMARY_RESPONSE_CACHE_PATH)
}

pub fn to_tmp_path(cache_path: &Path) -> PathBuf {
    cache_path.with_extension("tmp")
}

pub async fn read_jsonl_cache_stream<T>(cache_path: PathBuf) -> Result<impl Stream<Item = Result<T, ReadJsonlCacheStreamError>>, ReadJsonlCacheStreamError>
where
    T: for<'a> Deserialize<'a>,
{
    use ReadJsonlCacheStreamError::*;
    handle_bool!(!cache_path.exists(), CacheMissing, cache_path);
    let reader = handle!(Jsonl::from_path(&cache_path).await, OpenCacheFailed, cache_path);
    let cache_path_for_map = cache_path.clone();
    let stream = reader.deserialize::<T>().map(move |result| {
        result.map_err(|source| ParseCacheFailed {
            source,
            cache_path: cache_path_for_map.clone(),
        })
    });
    Ok(stream)
}

#[derive(Error, Debug)]
pub enum ParseBoolishError {
    #[error("invalid boolish value '{value}'")]
    InvalidValue { value: String },
}

#[derive(Error, Debug)]
pub enum ParseEnvVarError<E: StdError> {
    #[error("failed to read env var '{var}'")]
    ReadEnvVarFailed { source: env::VarError, var: String },
    #[error("failed to parse env var '{var}' with value '{value}'")]
    ParseValueFailed { source: E, var: String, value: String },
}

#[derive(Error, Debug)]
pub enum ReadJsonlCacheStreamError {
    #[error("cache not found at '{cache_path}'")]
    CacheMissing { cache_path: PathBuf },
    #[error("failed to open cache at '{cache_path}'")]
    OpenCacheFailed { source: anyhow::Error, cache_path: PathBuf },
    #[error("failed to parse cache at '{cache_path}'")]
    ParseCacheFailed { source: anyhow::Error, cache_path: PathBuf },
}
