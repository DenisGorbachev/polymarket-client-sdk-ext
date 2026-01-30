use crate::{CacheCheckCommand, CacheCheckCommandRunError, CacheDownloadCommand, CacheDownloadCommandRunError, CacheGammaEventsCommand, CacheGammaEventsCommandRunError, CacheMarketResponsesCommand, CacheMarketResponsesCommandRunError, CacheOrderBookSummaryResponsesCommand, CacheOrderBookSummaryResponsesCommandRunError};
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
    GammaEvents(CacheGammaEventsCommand),
    MarketResponses(CacheMarketResponsesCommand),
    OrderBookSummaryResponses(CacheOrderBookSummaryResponsesCommand),
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
            GammaEvents(command) => map_err!(command.run().await, CacheGammaEventsCommandRunFailed),
            MarketResponses(command) => map_err!(command.run().await, CacheMarketResponsesCommandRunFailed),
            OrderBookSummaryResponses(command) => map_err!(command.run().await, CacheOrderBookSummaryResponsesCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheCommandRunError {
    #[error("failed to run cache check command")]
    CacheCheckCommandRunFailed { source: CacheCheckCommandRunError },
    #[error("failed to run cache download command")]
    CacheDownloadCommandRunFailed { source: CacheDownloadCommandRunError },
    #[error("failed to run cache gamma events command")]
    CacheGammaEventsCommandRunFailed { source: CacheGammaEventsCommandRunError },
    #[error("failed to run cache market responses command")]
    CacheMarketResponsesCommandRunFailed { source: CacheMarketResponsesCommandRunError },
    #[error("failed to run cache order book summary responses command")]
    CacheOrderBookSummaryResponsesCommandRunFailed { source: CacheOrderBookSummaryResponsesCommandRunError },
}
