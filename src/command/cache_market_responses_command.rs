use CacheMarketResponsesSubcommand::*;
use errgonomic::map_err;
use std::process::ExitCode;
use thiserror::Error;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheMarketResponsesCommand {
    #[command(subcommand)]
    subcommand: CacheMarketResponsesSubcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum CacheMarketResponsesSubcommand {
    List(CacheMarketResponsesListCommand),
}

impl CacheMarketResponsesCommand {
    pub async fn run(self) -> Result<ExitCode, CacheMarketResponsesCommandRunError> {
        use CacheMarketResponsesCommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            List(command) => map_err!(command.run().await, CacheMarketResponsesListCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheMarketResponsesCommandRunError {
    #[error("failed to run cache market responses list command")]
    CacheMarketResponsesListCommandRunFailed { source: CacheMarketResponsesListCommandRunError },
}

mod cache_market_responses_list_command;

pub use cache_market_responses_list_command::*;
