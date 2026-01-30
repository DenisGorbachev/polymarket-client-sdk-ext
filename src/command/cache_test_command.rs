use crate::{CLOB_MARKET_RESPONSES_KEYSPACE, CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE, ConvertMarketResponseToMarketError, ConvertOrderBookSummaryResponseToOrderbookError, DEFAULT_DB_DIR, Market, OpenKeyspaceError, Orderbook, format_debug_diff, open_keyspace};
use errgonomic::{handle, handle_bool, map_err};
use fjall::{Readable, SingleWriterTxDatabase, Snapshot};
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
        let market_keyspace = handle!(open_keyspace(&db, CLOB_MARKET_RESPONSES_KEYSPACE), OpenMarketKeyspaceFailed);
        let orderbook_keyspace = handle!(open_keyspace(&db, CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE), OpenOrderbookKeyspaceFailed);
        let snapshot = db.read_tx();
        let batch_size = batch_size.get();
        handle!(
            Self::test_keyspace_round_trip::<MarketResponse, Market, ConvertMarketResponseToMarketError>(&snapshot, &market_keyspace, batch_size),
            TestKeyspaceRoundTripFailed,
            keyspace: CLOB_MARKET_RESPONSES_KEYSPACE.to_string(),
            batch_size
        );
        handle!(
            Self::test_keyspace_round_trip::<OrderBookSummaryResponse, Orderbook, ConvertOrderBookSummaryResponseToOrderbookError>(&snapshot, &orderbook_keyspace, batch_size),
            TestKeyspaceRoundTripFailed,
            keyspace: CLOB_ORDER_BOOK_SUMMARY_RESPONSE_KEYSPACE.to_string(),
            batch_size
        );
        Ok(ExitCode::SUCCESS)
    }

    fn test_keyspace_round_trip<T, U, E>(snapshot: &Snapshot, keyspace: &impl AsRef<fjall::Keyspace>, batch_size: usize) -> Result<(), CacheTestCommandRoundTripEntryError<T, E>>
    where
        T: DeserializeOwned + Clone + PartialEq + Send + Sync + core::fmt::Debug + 'static,
        U: TryFrom<T, Error = E>,
        T: From<U>,
        E: StdError + Send + Sync + 'static,
    {
        let iter = snapshot.iter(keyspace);
        iter.chunks(batch_size).into_iter().try_for_each(|chunk| {
            let values = match Self::collect_values::<T, E>(chunk) {
                Ok(values) => values,
                Err(error) => return Err(error),
            };
            values
                .par_iter()
                .try_for_each(Self::round_trip_value::<T, U, E>)
        })
    }

    fn collect_values<T, E>(guards: impl IntoIterator<Item = fjall::Guard>) -> Result<Vec<fjall::Slice>, CacheTestCommandRoundTripEntryError<T, E>>
    where
        E: StdError + Send + Sync + 'static,
    {
        use CacheTestCommandRoundTripEntryError::*;
        guards
            .into_iter()
            .map(|guard| map_err!(guard.into_inner(), ReadEntryFailed).map(|(_key, value)| value))
            .collect()
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
            bitcode::deserialize::<T>(value.as_ref()),
            DeserializeFailed,
            value: value.clone()
        );
        let output = handle!(U::try_from(input.clone()), TryFromFailed, input: Box::new(input));
        let input_round_trip = T::from(output);
        handle_bool!(
            input != input_round_trip,
            RoundTripFailed,
            diff: format_debug_diff(&input, &input_round_trip, "input", "input_round_trip"),
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
    #[error("failed to open market keyspace")]
    OpenMarketKeyspaceFailed { source: OpenKeyspaceError },
    #[error("failed to open orderbook keyspace")]
    OpenOrderbookKeyspaceFailed { source: OpenKeyspaceError },
    #[error("failed to test keyspace '{keyspace}' with batch size '{batch_size}'")]
    TestKeyspaceRoundTripFailed { source: Box<dyn StdError>, keyspace: String, batch_size: usize },
}

#[derive(Error, Debug)]
pub enum CacheTestCommandRoundTripEntryError<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    #[error("failed to read cache entry")]
    ReadEntryFailed { source: fjall::Error },
    #[error("failed to deserialize cache entry")]
    DeserializeFailed { source: bitcode::Error, value: fjall::Slice },
    #[error("failed to convert cache entry")]
    TryFromFailed { source: E, input: Box<T> },
    #[error("round-tripped cache entry does not match original: '{diff}'")]
    RoundTripFailed { input: Box<T>, input_round_trip: Box<T>, diff: String },
}
