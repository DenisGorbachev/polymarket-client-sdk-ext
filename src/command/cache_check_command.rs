use crate::{CLOB_MARKET_RESPONSES_KEYSPACE, ClobMarketResponsePrecise, DEFAULT_DB_DIR, MARKET_RESPONSE_PROPERTIES, OpenKeyspaceError, Property, PropertyName, PropertyStats, open_keyspace};
use errgonomic::{handle, handle_iter};
use fjall::{Readable, SingleWriterTxDatabase, Snapshot, UserKey};
use polymarket_client_sdk::clob::types::response::MarketResponse;
use rustc_hash::FxHashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

type ViolationStatsMap = FxHashMap<PropertyName, PropertyStats<3, String>>;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheCheckCommand {
    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,
}

impl CacheCheckCommand {
    pub async fn run(self) -> Result<ExitCode, CacheCheckCommandRunError> {
        use CacheCheckCommandRunError::*;
        let Self {
            dir,
        } = self;
        let db = handle!(SingleWriterTxDatabase::builder(&dir).open(), OpenDatabaseFailed, dir);
        let clob_market_responses_keyspace = handle!(open_keyspace(&db, CLOB_MARKET_RESPONSES_KEYSPACE), OpenMarketKeyspaceFailed);
        let snapshot = db.read_tx();
        let mut properties = Self::named_properties();
        let mut violations = Self::init_violations(&properties);
        let iter = snapshot.iter(&clob_market_responses_keyspace);
        let _processed = handle_iter!(iter.map(|guard| Self::process_entry(&mut violations, &mut properties, &snapshot, guard)), ProcessMarketEntryFailed);
        handle!(Self::write_violations(&violations), WriteViolationsFailed);
        Ok(ExitCode::SUCCESS)
    }

    fn named_properties() -> Vec<(PropertyName, Box<dyn Property<MarketResponse>>)> {
        MARKET_RESPONSE_PROPERTIES
            .into_iter()
            .map(|factory| {
                let property = factory();
                let name = property.name();
                (name, property)
            })
            .collect()
    }

    fn init_violations(properties: &[(PropertyName, Box<dyn Property<MarketResponse>>)]) -> ViolationStatsMap {
        properties
            .iter()
            .map(|(name, _)| (name.clone(), PropertyStats::default()))
            .collect()
    }

    fn process_entry(violations: &mut ViolationStatsMap, properties: &mut [(PropertyName, Box<dyn Property<MarketResponse>>)], snapshot: &Snapshot, guard: fjall::Guard) -> Result<(), CacheCheckCommandProcessMarketEntryError> {
        use CacheCheckCommandProcessMarketEntryError::*;
        let (key_slice, value_slice) = handle!(guard.into_inner(), ReadEntryFailed);
        let value = handle!(rkyv::from_bytes::<ClobMarketResponsePrecise, rkyv::rancor::Error>(value_slice.as_ref()), DeserializeFailed, value: value_slice);
        let market_response = MarketResponse::from(value);
        Self::record_violations(violations, properties, snapshot, key_slice, &market_response);
        Ok(())
    }

    fn record_violations<T>(violations: &mut ViolationStatsMap, properties: &mut [(PropertyName, Box<dyn Property<T>>)], snapshot: &Snapshot, key: UserKey, value: &T) {
        properties.iter_mut().for_each(|(name, property)| {
            if !property.holds(value, snapshot) {
                let stats = violations.entry(name.clone()).or_default();
                // TODO: Fix error handling
                let key_string = String::from_utf8(key.to_vec()).expect("every key should be a valid string");
                stats.witness(key_string);
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
    #[error("failed to open market keyspace")]
    OpenMarketKeyspaceFailed { source: OpenKeyspaceError },
    #[error("failed to process {len} cache entries", len = source.len())]
    ProcessMarketEntryFailed { source: errgonomic::ErrVec<CacheCheckCommandProcessMarketEntryError> },
    #[error("failed to write violations output")]
    WriteViolationsFailed { source: CacheCheckCommandWriteViolationsError },
}

#[derive(Error, Debug)]
pub enum CacheCheckCommandProcessMarketEntryError {
    #[error("failed to read cache entry")]
    ReadEntryFailed { source: fjall::Error },
    #[error("failed to deserialize market response")]
    DeserializeFailed { source: rkyv::rancor::Error, value: fjall::Slice },
}

#[derive(Error, Debug)]
pub enum CacheCheckCommandWriteViolationsError {
    #[error("failed to serialize violations output")]
    SerializeFailed { source: serde_json::Error },
    #[error("failed to write violations output")]
    WriteFailed { source: std::io::Error },
}
