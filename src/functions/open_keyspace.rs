use errgonomic::handle;
use fjall::{KeyspaceCreateOptions, SingleWriterTxDatabase, SingleWriterTxKeyspace};
use thiserror::Error;

/// Opens a Fjall keyspace with default options.
pub fn open_keyspace(db: &SingleWriterTxDatabase, keyspace: &'static str) -> Result<SingleWriterTxKeyspace, OpenKeyspaceError> {
    use OpenKeyspaceError::*;
    let keyspace_handle = handle!(db.keyspace(keyspace, KeyspaceCreateOptions::default), OpenKeyspaceFailed, keyspace);
    Ok(keyspace_handle)
}

/// Errors returned by [`open_keyspace`].
#[derive(Error, Debug)]
pub enum OpenKeyspaceError {
    #[error("failed to open keyspace '{keyspace}'")]
    OpenKeyspaceFailed { source: fjall::Error, keyspace: &'static str },
}
