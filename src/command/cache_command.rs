use crate::{CacheCheckCommand, CacheCheckCommandRunError, CacheDownloadCommand, CacheDownloadCommandRunError, CacheMarketResponsesCommand, CacheMarketResponsesCommandRunError, CacheOrderBookSummaryResponsesCommand, CacheOrderBookSummaryResponsesCommandRunError, CacheTestCommand, CacheTestCommandRunError};
use CacheSubcommand::*;
use errgonomic::map_err;
use std::process::ExitCode;
use thiserror::Error;

pub const DEFAULT_DB_DIR: &str = ".cache/db";

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheCommand {
    #[command(subcommand)]
    subcommand: CacheSubcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum CacheSubcommand {
    Check(CacheCheckCommand),
    Download(CacheDownloadCommand),
    MarketResponses(CacheMarketResponsesCommand),
    OrderBookSummaryResponses(CacheOrderBookSummaryResponsesCommand),
    Test(CacheTestCommand),
}

impl CacheCommand {
    pub async fn run(self) -> Result<ExitCode, CacheCommandRunError> {
        use CacheCommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            Check(command) => map_err!(command.run().await, CacheCheckCommandRunFailed),
            Download(command) => map_err!(command.run().await, CacheDownloadCommandRunFailed),
            MarketResponses(command) => map_err!(command.run().await, CacheMarketResponsesCommandRunFailed),
            OrderBookSummaryResponses(command) => map_err!(command.run().await, CacheOrderBookSummaryResponsesCommandRunFailed),
            Test(command) => map_err!(command.run().await, CacheTestCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheCommandRunError {
    #[error("failed to run cache check command")]
    CacheCheckCommandRunFailed { source: CacheCheckCommandRunError },
    #[error("failed to run cache download command")]
    CacheDownloadCommandRunFailed { source: CacheDownloadCommandRunError },
    #[error("failed to run cache market responses command")]
    CacheMarketResponsesCommandRunFailed { source: CacheMarketResponsesCommandRunError },
    #[error("failed to run cache order book summary responses command")]
    CacheOrderBookSummaryResponsesCommandRunFailed { source: CacheOrderBookSummaryResponsesCommandRunError },
    #[error("failed to run cache test command")]
    CacheTestCommandRunFailed { source: CacheTestCommandRunError },
}
