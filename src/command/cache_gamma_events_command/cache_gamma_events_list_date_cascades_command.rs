use crate::{DEFAULT_DB_DIR, GAMMA_EVENTS_KEYSPACE, OpenKeyspaceError, OutputKind, is_date_cascade, open_keyspace};
use errgonomic::handle;
use fjall::{Readable, SingleWriterTxDatabase};
use polymarket_client_sdk::gamma::types::response::Event;
use std::io::{Write, stdout};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

const KEY_VALUE_SEPARATOR: &str = ": ";

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheGammaEventsListDateCascadesCommand {
    #[arg(long, default_value = DEFAULT_DB_DIR)]
    pub dir: PathBuf,

    #[arg(long)]
    pub offset: Option<usize>,

    #[arg(long)]
    pub limit: Option<NonZeroUsize>,

    #[arg(long, value_enum, default_value_t = OutputKind::KeyValue)]
    pub kind: OutputKind,
}

impl CacheGammaEventsListDateCascadesCommand {
    pub async fn run(self) -> Result<ExitCode, CacheGammaEventsListDateCascadesCommandRunError> {
        use CacheGammaEventsListDateCascadesCommandRunError::*;
        let Self {
            dir,
            offset,
            limit,
            kind,
        } = self;
        let db = handle!(SingleWriterTxDatabase::builder(&dir).open(), OpenDatabaseFailed, dir);
        let keyspace = handle!(open_keyspace(&db, GAMMA_EVENTS_KEYSPACE), OpenKeyspaceFailed);
        let snapshot = db.read_tx();
        let offset = offset.unwrap_or(0);
        let limit = limit.map(NonZeroUsize::get).unwrap_or(usize::MAX);
        let iter = snapshot.iter(&keyspace);
        let mut stdout = stdout().lock();
        handle!(Self::write_date_cascade_events(iter, &mut stdout, kind, offset, limit), WriteDateCascadesFailed);
        Ok(ExitCode::SUCCESS)
    }

    fn write_date_cascade_events(iter: impl IntoIterator<Item = fjall::Guard>, writer: &mut impl Write, kind: OutputKind, offset: usize, limit: usize) -> Result<(), CacheGammaEventsListDateCascadesCommandWriteDateCascadesError> {
        use CacheGammaEventsListDateCascadesCommandWriteDateCascadesError::*;
        let mut entries = iter
            .into_iter()
            .map(Self::date_cascade_entry_from_guard)
            .filter_map(|result| match result {
                Ok(Some(entry)) => Some(Ok(entry)),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            })
            .skip(offset)
            .take(limit);
        entries.try_for_each(|entry| {
            let (key, value) = match entry {
                Ok(entry) => entry,
                Err(error) => return Err(error),
            };
            handle!(kind.write(writer, key.as_ref(), value.as_ref(), KEY_VALUE_SEPARATOR), WriteFailed);
            handle!(writer.write_all(b"\n"), WriteAllFailed);
            Ok(())
        })
    }

    fn date_cascade_entry_from_guard(guard: fjall::Guard) -> Result<Option<(fjall::Slice, fjall::Slice)>, CacheGammaEventsListDateCascadesCommandWriteDateCascadesError> {
        use CacheGammaEventsListDateCascadesCommandWriteDateCascadesError::*;
        let (key_slice, value_slice) = handle!(guard.into_inner(), ReadEntryFailed);
        let event = handle!(
            serde_json::from_slice::<Event>(value_slice.as_ref()),
            DeserializeFailed,
            value: value_slice
        );
        let is_date_cascade = is_date_cascade(&event).is_some_and(|value| value);
        if is_date_cascade { Ok(Some((key_slice, value_slice))) } else { Ok(None) }
    }
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsListDateCascadesCommandRunError {
    #[error("failed to open database at '{dir}'")]
    OpenDatabaseFailed { source: fjall::Error, dir: PathBuf },
    #[error("failed to open gamma events keyspace")]
    OpenKeyspaceFailed { source: OpenKeyspaceError },
    #[error("failed to write date cascade events output")]
    WriteDateCascadesFailed { source: CacheGammaEventsListDateCascadesCommandWriteDateCascadesError },
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsListDateCascadesCommandWriteDateCascadesError {
    #[error("failed to read cache entry")]
    ReadEntryFailed { source: fjall::Error },
    #[error("failed to deserialize event entry")]
    DeserializeFailed { source: serde_json::Error, value: fjall::Slice },
    #[error("failed to write output")]
    WriteFailed { source: crate::OutputKindWriteError },
    #[error("failed to write output newline")]
    WriteAllFailed { source: std::io::Error },
}
