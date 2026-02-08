use crate::{DEFAULT_DB_DIR, GAMMA_EVENTS_KEYSPACE, GAMMA_EVENTS_PAGE_SIZE, GammaEvent, OpenKeyspaceError, open_keyspace};
use errgonomic::{ErrVec, handle, handle_bool, handle_iter};
use fjall::{PersistMode, Readable, SingleWriterTxDatabase, SingleWriterTxKeyspace};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::EventsRequest;
use polymarket_client_sdk::gamma::types::response::Event;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheGammaEventsMonitorDateCascadesCommand {
    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,

    #[arg(long)]
    pub max_iterations: Option<NonZeroUsize>,
}

impl CacheGammaEventsMonitorDateCascadesCommand {
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
            handle!(Self::refresh_date_cascades(&db, &keyspace, &client, &event_ids).await, RefreshDateCascadesFailed);
            iterations = iterations.saturating_add(1);
            if max_iterations.is_some_and(|max_iterations| iterations >= max_iterations) {
                break;
            }
        }
        Ok(ExitCode::SUCCESS)
    }

    fn collect_date_cascade_event_ids(db: &SingleWriterTxDatabase, keyspace: &SingleWriterTxKeyspace) -> Result<Vec<String>, CacheGammaEventsMonitorDateCascadesCommandCollectDateCascadeEventIdsError> {
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

    fn date_cascade_event_id_from_guard(guard: fjall::Guard) -> Result<Option<String>, CacheGammaEventsMonitorDateCascadesCommandDateCascadeEventIdFromGuardError> {
        use CacheGammaEventsMonitorDateCascadesCommandDateCascadeEventIdFromGuardError::*;
        let (_key, value) = handle!(guard.into_inner(), ReadEntryFailed);
        let event = handle!(rkyv::from_bytes::<GammaEvent, rkyv::rancor::Error>(value.as_ref()), DeserializeFailed, value);
        let is_date_cascade = event.is_date_cascade().is_some_and(|value| value);
        if is_date_cascade {
            handle_bool!(event.id.trim().is_empty(), EventIdInvalid, event: Box::new(event));
            Ok(Some(event.id))
        } else {
            Ok(None)
        }
    }

    async fn refresh_date_cascades(db: &SingleWriterTxDatabase, keyspace: &SingleWriterTxKeyspace, client: &GammaClient, event_ids: &[String]) -> Result<(), CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesError> {
        use CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesError::*;
        for chunk in event_ids.chunks(GAMMA_EVENTS_PAGE_SIZE) {
            let events_len = handle!(Self::refresh_date_cascade_chunk(db, keyspace, client, chunk).await, RefreshDateCascadesChunkFailed);
            println!("{events_len}");
        }
        Ok(())
    }

    async fn refresh_date_cascade_chunk(db: &SingleWriterTxDatabase, keyspace: &SingleWriterTxKeyspace, client: &GammaClient, event_ids: &[String]) -> Result<usize, CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesChunkError> {
        use CacheGammaEventsMonitorDateCascadesCommandRefreshDateCascadesChunkError::*;
        let request = EventsRequest::builder()
            .id(event_ids.to_vec())
            .order(vec!["id".to_string()])
            .limit(GAMMA_EVENTS_PAGE_SIZE as i32)
            .ascending(true)
            .build();
        let events = handle!(client.events(&request).await, EventsFailed, request: Box::new(request));
        let events_len = events.len();
        handle!(Self::write_events_to_database(db, keyspace, events), WriteEventsToDatabaseFailed);
        Ok(events_len)
    }

    fn write_events_to_database(db: &SingleWriterTxDatabase, keyspace: &SingleWriterTxKeyspace, events: Vec<Event>) -> Result<(), CacheGammaEventsMonitorDateCascadesCommandWriteEventsToDatabaseError> {
        use CacheGammaEventsMonitorDateCascadesCommandWriteEventsToDatabaseError::*;
        let serialized_events = handle_iter!(events.into_iter().map(Self::serialize_event_entry), SerializeEventEntryFailed);
        let mut tx = db.write_tx();
        serialized_events.into_iter().for_each(|(event_id, bytes)| {
            tx.insert(keyspace, event_id, bytes);
        });
        handle!(tx.commit(), CommitTransactionFailed);
        handle!(db.persist(PersistMode::SyncAll), PersistDatabaseFailed);
        Ok(())
    }

    fn serialize_event_entry(event: Event) -> Result<(String, Vec<u8>), CacheGammaEventsMonitorDateCascadesCommandSerializeEventEntryError> {
        use CacheGammaEventsMonitorDateCascadesCommandSerializeEventEntryError::*;
        let event = handle!(GammaEvent::try_from(event), TryFromFailed);
        let event_id = event.id.clone();
        let bytes = handle!(rkyv::to_bytes::<rkyv::rancor::Error>(&event), SerializeFailed, event: Box::new(event));
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
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandCollectDateCascadeEventIdsError {
    #[error("failed to process '{len}' date cascade events", len = source.len())]
    DateCascadeEventIdFromGuardFailed { source: ErrVec<CacheGammaEventsMonitorDateCascadesCommandDateCascadeEventIdFromGuardError> },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandDateCascadeEventIdFromGuardError {
    #[error("failed to read cache entry")]
    ReadEntryFailed { source: fjall::Error },
    #[error("failed to deserialize event entry")]
    DeserializeFailed { source: rkyv::rancor::Error, value: fjall::Slice },
    #[error("event response has empty event id")]
    EventIdInvalid { event: Box<GammaEvent> },
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
    #[error("failed to persist events to database")]
    WriteEventsToDatabaseFailed { source: CacheGammaEventsMonitorDateCascadesCommandWriteEventsToDatabaseError },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandWriteEventsToDatabaseError {
    #[error("failed to serialize '{len}' event responses", len = source.len())]
    SerializeEventEntryFailed { source: ErrVec<CacheGammaEventsMonitorDateCascadesCommandSerializeEventEntryError> },
    #[error("failed to commit database transaction")]
    CommitTransactionFailed { source: fjall::Error },
    #[error("failed to persist database changes")]
    PersistDatabaseFailed { source: fjall::Error },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsMonitorDateCascadesCommandSerializeEventEntryError {
    #[error("failed to convert gamma event response")]
    TryFromFailed { source: crate::ConvertGammaEventRawToGammaEventError },
    #[error("failed to serialize event response")]
    SerializeFailed { source: rkyv::rancor::Error, event: Box<GammaEvent> },
}
