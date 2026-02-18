use crate::{DEFAULT_DB_DIR, GAMMA_EVENTS_KEYSPACE, GAMMA_EVENTS_PAGE_SIZE, GammaEvent, GammaEventGetTimeSpreadArbitrageOpportunitiesError, OpenKeyspaceError, open_keyspace};
use errgonomic::{ErrVec, handle, handle_iter};
use fjall::{PersistMode, Readable, SingleWriterTxDatabase, SingleWriterTxKeyspace};
use itertools::Itertools;
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::EventsRequest;
use std::io::{Write, stdout};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheGammaEventsMonitorTimeSpreadOpportunitiesCommand {
    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,

    #[arg(long)]
    pub max_iterations: Option<NonZeroUsize>,
}

impl CacheGammaEventsMonitorTimeSpreadOpportunitiesCommand {
    pub async fn run(self) -> Result<ExitCode, CacheGammaEventsMonitorDateCascadesCommandRunError> {
        use CacheGammaEventsMonitorDateCascadesCommandRunError::*;
        let Self {
            dir,
            max_iterations,
        } = self;
        let db = handle!(SingleWriterTxDatabase::builder(&dir).open(), OpenDatabaseFailed, dir);
        let keyspace = handle!(open_keyspace(&db, GAMMA_EVENTS_KEYSPACE), OpenKeyspaceFailed);
        let event_ids = handle!(Self::collect_date_cascade_event_ids(&db, &keyspace), CollectDateCascadeEventIdsFailed);
        let client = GammaClient::default();
        let max_iterations = max_iterations.map(NonZeroUsize::get);
        let mut iterations = 0usize;
        loop {
            let events = handle!(Self::refresh_date_cascades(&db, &keyspace, &client, &event_ids).await, RefreshDateCascadesFailed);
            let opportunities = handle_iter!(
                events
                    .iter()
                    .map(|event| event.get_time_spread_arbitrage_opportunities()),
                GetTimeSpreadArbitrageOpportunitiesFailed
            );
            let mut stdout = stdout().lock();
            let opportunities_print_results = opportunities.into_iter().flatten().map(|opportunity| {
                serde_json::ser::to_writer(&mut stdout, &opportunity)?;
                stdout.write_all(b"\n")
            });
            let opportunities_print_result: Result<(), _> = opportunities_print_results.try_collect();
            opportunities_print_result.unwrap();
            iterations = iterations.saturating_add(1);
            if max_iterations.is_some_and(|max_iterations| iterations >= max_iterations) {
                break;
            }
        }
        Ok(ExitCode::SUCCESS)
    }

    fn collect_date_cascade_event_ids(db: &SingleWriterTxDatabase, keyspace: &SingleWriterTxKeyspace) -> Result<Vec<u64>, CacheGammaEventsMonitorDateCascadesCommandCollectDateCascadeEventIdsError> {
        use CacheGammaEventsMonitorDateCascadesCommandCollectDateCascadeEventIdsError::*;
        let snapshot = db.read_tx();
        let iter = snapshot.iter(keyspace);
        let results = iter
            .into_iter()
            .map(Self::date_cascade_event_id_from_guard)
            .filter_map(|result| match result {
                Ok(Some(event_id)) => Some(Ok(event_id)),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            });
        let event_ids = handle_iter!(results, DateCascadeEventIdFromGuardFailed);
        Ok(event_ids)
    }

    fn date_cascade_event_id_from_guard(guard: fjall::Guard) -> Result<Option<u64>, CacheGammaEventsMonitorDateCascadesCommandDateCascadeEventIdFromGuardError> {
        use CacheGammaEventsMonitorDateCascadesCommandDateCascadeEventIdFromGuardError::*;
        let (_key, value) = handle!(guard.into_inner(), ReadEntryFailed);
        let event = handle!(rkyv::from_bytes::<GammaEvent, rkyv::rancor::Error>(value.as_ref()), DeserializeFailed, value);
        if event.is_date_cascade.unwrap_or_default() { Ok(Some(event.id)) } else { Ok(None) }
    }

    async fn refresh_date_cascades(db: &SingleWriterTxDatabase, keyspace: &SingleWriterTxKeyspace, client: &GammaClient, event_ids: &[u64]) -> Result<Vec<GammaEvent>, CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesError> {
        use CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesError::*;
        let mut events_all = Vec::new();
        for chunk in event_ids.chunks(GAMMA_EVENTS_PAGE_SIZE) {
            let mut events_chunk = handle!(Self::refresh_date_cascade_chunk(db, keyspace, client, chunk).await, RefreshDateCascadesChunkFailed);
            events_all.append(&mut events_chunk);
        }
        Ok(events_all)
    }

    async fn refresh_date_cascade_chunk(db: &SingleWriterTxDatabase, keyspace: &SingleWriterTxKeyspace, client: &GammaClient, event_ids: &[u64]) -> Result<Vec<GammaEvent>, CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesChunkError> {
        use CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesChunkError::*;
        let request = EventsRequest::builder()
            .id(event_ids.iter().map(ToString::to_string).collect())
            .order(vec!["id".to_string()])
            .limit(GAMMA_EVENTS_PAGE_SIZE as i32)
            .ascending(true)
            .build();
        let events_raw = handle!(client.events(&request).await, EventsFailed, request: Box::new(request));
        let events = handle_iter!(events_raw.into_iter().map(GammaEvent::try_from), TryFromFailed);
        handle!(Self::write_events_to_database(db, keyspace, &events), WriteEventsToDatabaseFailed);
        Ok(events)
    }

    fn write_events_to_database(db: &SingleWriterTxDatabase, keyspace: &SingleWriterTxKeyspace, events: &[GammaEvent]) -> Result<(), CacheGammaEventsMonitorDateCascadesCommandWriteEventsToDatabaseError> {
        use CacheGammaEventsMonitorDateCascadesCommandWriteEventsToDatabaseError::*;
        let serialized_events = handle_iter!(events.iter().map(Self::serialize_event_entry), SerializeEventEntryFailed);
        let mut tx = db.write_tx();
        serialized_events.into_iter().for_each(|(event_id, bytes)| {
            tx.insert(keyspace, event_id, bytes);
        });
        handle!(tx.commit(), CommitTransactionFailed);
        handle!(db.persist(PersistMode::SyncAll), PersistDatabaseFailed);
        Ok(())
    }

    fn serialize_event_entry(event: &GammaEvent) -> Result<(String, Vec<u8>), CacheGammaEventsMonitorDateCascadesCommandSerializeEventEntryError> {
        use CacheGammaEventsMonitorDateCascadesCommandSerializeEventEntryError::*;
        let event_id = event.id.to_string();
        let bytes = handle!(rkyv::to_bytes::<rkyv::rancor::Error>(event), SerializeFailed, event_id);
        Ok((event_id, bytes.into_vec()))
    }
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandRunError {
    #[error("failed to open database at '{dir}'")]
    OpenDatabaseFailed { source: fjall::Error, dir: PathBuf },
    #[error("failed to open gamma events keyspace")]
    OpenKeyspaceFailed { source: OpenKeyspaceError },
    #[error("failed to collect date cascade event ids")]
    CollectDateCascadeEventIdsFailed { source: CacheGammaEventsMonitorDateCascadesCommandCollectDateCascadeEventIdsError },
    #[error("failed to refresh date cascade events")]
    RefreshDateCascadesFailed { source: CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesError },
    #[error("failed to compute time spread arbitrage opportunities")]
    GetTimeSpreadArbitrageOpportunitiesFailed { source: ErrVec<GammaEventGetTimeSpreadArbitrageOpportunitiesError> },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandCollectDateCascadeEventIdsError {
    #[error("failed to process {len} date cascade events", len = source.len())]
    DateCascadeEventIdFromGuardFailed { source: ErrVec<CacheGammaEventsMonitorDateCascadesCommandDateCascadeEventIdFromGuardError> },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandDateCascadeEventIdFromGuardError {
    #[error("failed to read cache entry")]
    ReadEntryFailed { source: fjall::Error },
    #[error("failed to deserialize event entry")]
    DeserializeFailed { source: rkyv::rancor::Error, value: fjall::Slice },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesError {
    #[error("failed to refresh date cascade events chunk")]
    RefreshDateCascadesChunkFailed { source: CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesChunkError },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesChunkError {
    #[error("failed to fetch gamma events")]
    EventsFailed { source: polymarket_client_sdk::error::Error, request: Box<EventsRequest> },
    #[error("failed to convert {len} gamma event responses", len = source.len())]
    TryFromFailed { source: ErrVec<crate::ConvertGammaEventRawToGammaEventError> },
    #[error("failed to persist events to database")]
    WriteEventsToDatabaseFailed { source: CacheGammaEventsMonitorDateCascadesCommandWriteEventsToDatabaseError },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandWriteEventsToDatabaseError {
    #[error("failed to serialize {len} event responses", len = source.len())]
    SerializeEventEntryFailed { source: ErrVec<CacheGammaEventsMonitorDateCascadesCommandSerializeEventEntryError> },
    #[error("failed to commit database transaction")]
    CommitTransactionFailed { source: fjall::Error },
    #[error("failed to persist database changes")]
    PersistDatabaseFailed { source: fjall::Error },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandSerializeEventEntryError {
    #[error("failed to serialize event response for event '{event_id}'")]
    SerializeFailed { source: rkyv::rancor::Error, event_id: String },
}
