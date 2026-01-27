use crate::{CacheDownloadCommand, CacheDownloadCommandRunError, CacheMarketResponsesCommand, CacheMarketResponsesCommandRunError, CacheOrderBookSummaryResponsesCommand, CacheOrderBookSummaryResponsesCommandRunError};
use CacheSubcommand::*;
use errgonomic::map_err;
use thiserror::Error;

#[macro_export]
macro_rules! define_cache_list_command {
    (
        command = $command:ident,
        run_error = $run_error:ident,
        process_error = $process_error:ident,
        keyspace_const = $keyspace_const:ident,
    ) => {
        #[derive(clap::Parser, Clone, Debug)]
        pub struct $command {
            #[arg(long, default_value = $crate::DEFAULT_DB_DIR)]
            pub dir: std::path::PathBuf,
            #[arg(long)]
            pub limit: Option<std::num::NonZeroUsize>,
        }

        impl $command {
            pub async fn run(self) -> Result<(), $run_error> {
                use $run_error::*;
                use fjall::Readable;
                let Self { dir, limit } = self;
                let db = errgonomic::handle!(
                    fjall::SingleWriterTxDatabase::builder(&dir).open(),
                    OpenDatabaseFailed,
                    dir
                );
                let keyspace = errgonomic::handle!(
                    db.keyspace($crate::$keyspace_const, fjall::KeyspaceCreateOptions::default),
                    OpenKeyspaceFailed,
                    keyspace: $crate::$keyspace_const.to_string()
                );
                let read_tx = db.read_tx();
                let limit = limit.map(std::num::NonZeroUsize::get).unwrap_or(usize::MAX);
                let iter = read_tx.iter(&keyspace).take(limit);
                let mut stdout = std::io::stdout().lock();
                let _writes = errgonomic::handle_iter!(
                    iter.map(|guard| Self::process_entry(&mut stdout, guard)),
                    ProcessEntryFailed
                );
                Ok(())
            }

            fn process_entry(writer: &mut dyn std::io::Write, guard: fjall::Guard) -> Result<(), $process_error> {
                use $process_error::*;
                let (_key, value) = errgonomic::handle!(guard.into_inner(), ReadEntryFailed);
                errgonomic::handle!(writer.write_all(value.as_ref()), WriteFailed);
                errgonomic::handle!(writer.write_all(b"\n"), WriteFailed);
                Ok(())
            }
        }

        #[derive(thiserror::Error, Debug)]
        pub enum $run_error {
            #[error("failed to open database at '{dir}'")]
            OpenDatabaseFailed { source: fjall::Error, dir: std::path::PathBuf },
            #[error("failed to open keyspace '{keyspace}'")]
            OpenKeyspaceFailed { source: fjall::Error, keyspace: String },
            #[error("failed to process '{len}' cache entries", len = source.len())]
            ProcessEntryFailed { source: errgonomic::ErrVec<$process_error> },
        }

        #[derive(thiserror::Error, Debug)]
        pub enum $process_error {
            #[error("failed to read cache entry")]
            ReadEntryFailed { source: fjall::Error },
            #[error("failed to write cache entry")]
            WriteFailed { source: std::io::Error },
        }
    };
}

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheCommand {
    #[command(subcommand)]
    subcommand: CacheSubcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum CacheSubcommand {
    Download(CacheDownloadCommand),
    MarketResponses(CacheMarketResponsesCommand),
    OrderBookSummaryResponses(CacheOrderBookSummaryResponsesCommand),
}

impl CacheCommand {
    pub async fn run(self) -> Result<(), CacheCommandRunError> {
        use CacheCommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            Download(command) => map_err!(command.run().await, CacheDownloadCommandRunFailed),
            MarketResponses(command) => map_err!(command.run().await, CacheMarketResponsesCommandRunFailed),
            OrderBookSummaryResponses(command) => map_err!(command.run().await, CacheOrderBookSummaryResponsesCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheCommandRunError {
    #[error("failed to run cache download command")]
    CacheDownloadCommandRunFailed { source: CacheDownloadCommandRunError },
    #[error("failed to run cache market responses command")]
    CacheMarketResponsesCommandRunFailed { source: CacheMarketResponsesCommandRunError },
    #[error("failed to run cache order book summary responses command")]
    CacheOrderBookSummaryResponsesCommandRunFailed { source: CacheOrderBookSummaryResponsesCommandRunError },
}
