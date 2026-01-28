use CacheOrderBookSummaryResponsesSubcommand::*;
use errgonomic::map_err;
use std::process::ExitCode;
use thiserror::Error;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheOrderBookSummaryResponsesCommand {
    #[command(subcommand)]
    subcommand: CacheOrderBookSummaryResponsesSubcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum CacheOrderBookSummaryResponsesSubcommand {
    List(CacheOrderBookSummaryResponsesListCommand),
}

impl CacheOrderBookSummaryResponsesCommand {
    pub async fn run(self) -> Result<ExitCode, CacheOrderBookSummaryResponsesCommandRunError> {
        use CacheOrderBookSummaryResponsesCommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            List(command) => map_err!(command.run().await.map(|_| ExitCode::SUCCESS), CacheOrderBookSummaryResponsesListCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheOrderBookSummaryResponsesCommandRunError {
    #[error("failed to run cache order book summary responses list command")]
    CacheOrderBookSummaryResponsesListCommandRunFailed { source: CacheOrderBookSummaryResponsesListCommandRunError },
}

mod cache_order_book_summary_responses_list_command;

pub use cache_order_book_summary_responses_list_command::*;
