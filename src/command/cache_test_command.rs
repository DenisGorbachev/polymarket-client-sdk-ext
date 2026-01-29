use crate::{CLOB_MARKET_RESPONSES_KEYSPACE, CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE, ConvertMarketResponseToMarketError, ConvertOrderBookSummaryResponseToOrderbookError, DEFAULT_DB_DIR, Market, Orderbook};
use errgonomic::{exit_iterator_of_results_print_first, handle, handle_bool};
use fjall::{KeyspaceCreateOptions, Readable, SingleWriterTxDatabase, Snapshot};
use itertools::Itertools;
use polymarket_client_sdk::clob::types::response::{MarketResponse, OrderBookSummaryResponse};
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use std::error::Error as StdError;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheTestCommand {
    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,

    #[arg(long, default_value = "1024")]
    pub batch_size: NonZeroUsize,
}

impl CacheTestCommand {
    pub async fn run(self) -> Result<ExitCode, CacheTestCommandRunError> {
        use CacheTestCommandRunError::*;
        let Self {
            dir,
            batch_size,
        } = self;
        let db = handle!(SingleWriterTxDatabase::builder(&dir).open(), OpenDatabaseFailed, dir);
        let market_keyspace = handle!(
            db.keyspace(CLOB_MARKET_RESPONSES_KEYSPACE, KeyspaceCreateOptions::default),
            OpenMarketKeyspaceFailed,
            keyspace: CLOB_MARKET_RESPONSES_KEYSPACE.to_string()
        );
        let orderbook_keyspace = handle!(
            db.keyspace(CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE, KeyspaceCreateOptions::default),
            OpenOrderbookKeyspaceFailed,
            keyspace: CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE.to_string()
        );
        let snapshot = db.read_tx();
        let batch_size = batch_size.get();
        let market_exit = Self::test_keyspace_round_trip::<MarketResponse, Market, ConvertMarketResponseToMarketError>(&snapshot, &market_keyspace, batch_size);
        let orderbook_exit = Self::test_keyspace_round_trip::<OrderBookSummaryResponse, Orderbook, ConvertOrderBookSummaryResponseToOrderbookError>(&snapshot, &orderbook_keyspace, batch_size);
        let exit_code = [market_exit, orderbook_exit]
            .into_iter()
            .find(|code| *code != ExitCode::SUCCESS)
            .unwrap_or(ExitCode::SUCCESS);
        Ok(exit_code)
    }

    fn test_keyspace_round_trip<T, U, E>(snapshot: &Snapshot, keyspace: &impl AsRef<fjall::Keyspace>, batch_size: usize) -> ExitCode
    where
        T: DeserializeOwned + Clone + PartialEq + Send + core::fmt::Debug + 'static,
        U: TryFrom<T, Error = E>,
        T: From<U>,
        E: StdError + Send + Sync + 'static,
    {
        let iter = snapshot.iter(keyspace);
        iter.chunks(batch_size)
            .into_iter()
            .map(|chunk| {
                let chunk = chunk.collect::<Vec<_>>();
                let (values, read_errors) = Self::collect_values::<T, E>(chunk);
                let read_exit = exit_iterator_of_results_print_first(read_errors.into_iter().map(Err));
                if read_exit != ExitCode::SUCCESS {
                    return read_exit;
                }
                let results = values
                    .par_iter()
                    .map(Self::round_trip_value::<T, U, E>)
                    .collect::<Vec<_>>();
                exit_iterator_of_results_print_first(results)
            })
            .find(|code| *code != ExitCode::SUCCESS)
            .unwrap_or(ExitCode::SUCCESS)
    }

    fn collect_values<T, E>(guards: Vec<fjall::Guard>) -> (Vec<fjall::Slice>, Vec<CacheTestCommandRoundTripEntryError<T, E>>)
    where
        E: StdError + Send + Sync + 'static,
    {
        use CacheTestCommandRoundTripEntryError::*;
        let mut values = Vec::new();
        let mut errors = Vec::new();
        guards
            .into_iter()
            .for_each(|guard| match guard.into_inner() {
                Ok((_key, value)) => values.push(value),
                Err(source) => errors.push(ReadEntryFailed {
                    source,
                }),
            });
        (values, errors)
    }

    fn round_trip_value<T, U, E>(value: &fjall::Slice) -> Result<(), CacheTestCommandRoundTripEntryError<T, E>>
    where
        T: DeserializeOwned + Clone + PartialEq + core::fmt::Debug,
        U: TryFrom<T, Error = E>,
        T: From<U>,
        E: StdError + Send + Sync + 'static,
    {
        use CacheTestCommandRoundTripEntryError::*;
        let input = handle!(
            serde_json::from_slice::<T>(value.as_ref()),
            DeserializeFailed,
            value: value.clone()
        );
        let output = handle!(U::try_from(input.clone()), TryFromFailed, input: Box::new(input));
        let input_round_trip = T::from(output);
        handle_bool!(
            input != input_round_trip,
            RoundTripFailed,
            input: Box::new(input),
            input_round_trip: Box::new(input_round_trip)
        );
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum CacheTestCommandRunError {
    #[error("failed to open database at '{dir}'")]
    OpenDatabaseFailed { source: fjall::Error, dir: PathBuf },
    #[error("failed to open market keyspace '{keyspace}'")]
    OpenMarketKeyspaceFailed { source: fjall::Error, keyspace: String },
    #[error("failed to open orderbook keyspace '{keyspace}'")]
    OpenOrderbookKeyspaceFailed { source: fjall::Error, keyspace: String },
}

#[derive(Error, Debug)]
pub enum CacheTestCommandRoundTripEntryError<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    #[error("failed to read cache entry")]
    ReadEntryFailed { source: fjall::Error },
    #[error("failed to deserialize cache entry")]
    DeserializeFailed { source: serde_json::Error, value: fjall::Slice },
    #[error("failed to convert cache entry")]
    TryFromFailed { source: E, input: Box<T> },
    #[error("round-tripped cache entry does not match original")]
    RoundTripFailed { input: Box<T>, input_round_trip: Box<T> },
}
