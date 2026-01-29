use std::process::ExitCode;
use thiserror::Error;
// use CacheMarketResponsesSubcommand::*;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheMarketResponsesCommand {
    #[command(subcommand)]
    subcommand: CacheMarketResponsesSubcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum CacheMarketResponsesSubcommand {}

impl CacheMarketResponsesCommand {
    pub async fn run(self) -> Result<ExitCode, CacheMarketResponsesCommandRunError> {
        // use CacheMarketResponsesCommandRunError::*;
        // let Self {
        //     subcommand: _,
        // } = self;
        Ok(ExitCode::SUCCESS)
    }
}

#[derive(Error, Debug)]
pub enum CacheMarketResponsesCommandRunError {}
