use async_jsonl::{Jsonl, JsonlDeserialize};
use errgonomic::{handle, handle_bool, map_err};
use futures::{Stream, StreamExt};
use serde::Deserialize;
use std::env::{VarError, var};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const MARKET_RESPONSE_PAGE_CACHE_LIMIT_ENV: &str = "MARKET_RESPONSE_PAGE_CACHE_LIMIT";
pub const CACHE_DIR: &str = ".cache";
pub const MARKET_RESPONSE_CACHE_PATH: &str = "market_response.all.jsonl";
pub const ORDERBOOK_SUMMARY_RESPONSE_CACHE_PATH: &str = "orderbook_summary_response.all.jsonl";

pub fn parse_env_var<K: AsRef<OsStr>, T, E>(key: K, parse: impl FnOnce(String) -> Result<T, E>) -> Result<Option<T>, ParseEnvVarError<E>> {
    use ParseEnvVarError::*;
    match var(key) {
        Ok(value) => map_err!(parse(value), ParseValueFailed).map(Some),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(value)) => Err(NotUnicode {
            value,
        }),
    }
}

pub fn parse_boolish(value: &str) -> Result<bool, ParseBoolishError> {
    use ParseBoolishError::*;
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "1" | "true" | "t" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "f" | "no" | "n" | "off" => Ok(false),
        _ => Err(InvalidValue),
    }
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
    #[error("invalid boolish value")]
    InvalidValue,
}

#[derive(Error, Debug)]
pub enum ParseEnvVarError<E> {
    #[error("env var is not Unicode: '{}'", value.to_string_lossy())]
    NotUnicode { value: OsString },
    #[error("failed to parse env var")]
    ParseValueFailed { source: E },
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
