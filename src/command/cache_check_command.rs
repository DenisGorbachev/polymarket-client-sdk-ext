use crate::{CLOB_MARKET_RESPONSE_KEYSPACE, DEFAULT_DB_DIR, Property, PropertyName, ViolationStats, market_response_properties};
use errgonomic::{handle, handle_iter};
use fjall::{KeyspaceCreateOptions, Readable, SingleWriterTxDatabase, Snapshot};
use polymarket_client_sdk::clob::types::response::MarketResponse;
use rustc_hash::FxHashMap;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

const VIOLATION_EXAMPLE_LIMIT: usize = 10;

type ViolationStatsMap = FxHashMap<PropertyName, ViolationStats<VIOLATION_EXAMPLE_LIMIT, String>>;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheCheckCommand {
    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,
}

impl CacheCheckCommand {
    pub async fn run(self) -> Result<(), CacheCheckCommandRunError> {
        use CacheCheckCommandRunError::*;
        let Self {
            dir,
        } = self;
        let db = handle!(SingleWriterTxDatabase::builder(&dir).open(), OpenDatabaseFailed, dir);
        let keyspace = handle!(
            db.keyspace(CLOB_MARKET_RESPONSE_KEYSPACE, KeyspaceCreateOptions::default),
            OpenMarketKeyspaceFailed,
            keyspace: CLOB_MARKET_RESPONSE_KEYSPACE.to_string()
        );
        let snapshot = db.read_tx();
        let mut properties = Self::named_properties();
        let mut violations = Self::init_violations(&properties);
        let iter = snapshot.iter(&keyspace);
        let _processed = handle_iter!(iter.map(|guard| Self::process_market_entry(&mut violations, &mut properties, &snapshot, guard)), ProcessMarketEntryFailed);
        handle!(Self::write_violations(&violations), WriteViolationsFailed);
        Ok(())
    }

    fn named_properties() -> Vec<(PropertyName, Box<dyn Property<MarketResponse>>)> {
        market_response_properties()
            .into_iter()
            .map(|property| {
                let name = property.name();
                (name, property)
            })
            .collect()
    }

    fn init_violations(properties: &[(PropertyName, Box<dyn Property<MarketResponse>>)]) -> ViolationStatsMap {
        properties
            .iter()
            .map(|(name, _)| (name.clone(), ViolationStats::default()))
            .collect()
    }

    fn process_market_entry(violations: &mut ViolationStatsMap, properties: &mut [(PropertyName, Box<dyn Property<MarketResponse>>)], snapshot: &Snapshot, guard: fjall::Guard) -> Result<(), CacheCheckCommandProcessMarketEntryError> {
        use CacheCheckCommandProcessMarketEntryError::*;
        let value = handle!(guard.value(), ReadEntryFailed);
        let market_response = handle!(serde_json::from_slice::<MarketResponse>(value.as_ref()), DeserializeFailed, value);
        Self::record_violations(violations, properties, snapshot, &market_response);
        Ok(())
    }

    fn record_violations(violations: &mut ViolationStatsMap, properties: &mut [(PropertyName, Box<dyn Property<MarketResponse>>)], snapshot: &Snapshot, market_response: &MarketResponse) {
        properties.iter_mut().for_each(|(name, property)| {
            if !property.holds(market_response, snapshot) {
                let stats = violations.entry(name.clone()).or_default();
                stats.witness(market_response.market_slug.clone());
            }
        });
    }

    fn write_violations(violations: &ViolationStatsMap) -> Result<(), CacheCheckCommandWriteViolationsError> {
        use CacheCheckCommandWriteViolationsError::*;
        let mut stdout = std::io::stdout().lock();
        handle!(serde_json::to_writer_pretty(&mut stdout, violations), SerializeFailed);
        handle!(stdout.write_all(b"\n"), WriteFailed);
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum CacheCheckCommandRunError {
    #[error("failed to open database at '{dir}'")]
    OpenDatabaseFailed { source: fjall::Error, dir: PathBuf },
    #[error("failed to open market keyspace '{keyspace}'")]
    OpenMarketKeyspaceFailed { source: fjall::Error, keyspace: String },
    #[error("failed to process '{len}' cache entries", len = source.len())]
    ProcessMarketEntryFailed { source: errgonomic::ErrVec<CacheCheckCommandProcessMarketEntryError> },
    #[error("failed to write violations output")]
    WriteViolationsFailed { source: CacheCheckCommandWriteViolationsError },
}

#[derive(Error, Debug)]
pub enum CacheCheckCommandProcessMarketEntryError {
    #[error("failed to read cache entry")]
    ReadEntryFailed { source: fjall::Error },
    #[error("failed to deserialize market response")]
    DeserializeFailed { source: serde_json::Error, value: fjall::Slice },
}

#[derive(Error, Debug)]
pub enum CacheCheckCommandWriteViolationsError {
    #[error("failed to serialize violations output")]
    SerializeFailed { source: serde_json::Error },
    #[error("failed to write violations output")]
    WriteFailed { source: std::io::Error },
}
