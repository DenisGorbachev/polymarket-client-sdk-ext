use std::process::ExitCode;
use thiserror::Error;
// use CacheOrderBookSummaryResponsesSubcommand::*;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheOrderBookSummaryResponsesCommand {
    #[command(subcommand)]
    subcommand: CacheOrderBookSummaryResponsesSubcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum CacheOrderBookSummaryResponsesSubcommand {}

impl CacheOrderBookSummaryResponsesCommand {
    pub async fn run(self) -> Result<ExitCode, CacheOrderBookSummaryResponsesCommandRunError> {
        // use CacheOrderBookSummaryResponsesCommandRunError::*;
        // let Self {
        //     subcommand,
        // } = self;
        // match subcommand {}
        Ok(ExitCode::SUCCESS)
    }
}

#[derive(Error, Debug)]
pub enum CacheOrderBookSummaryResponsesCommandRunError {}
