use CacheGammaEventsSubcommand::*;
use errgonomic::map_err;
use std::process::ExitCode;
use thiserror::Error;

#[derive(clap::Parser, Clone, Debug)]
pub struct CacheGammaEventsCommand {
    #[command(subcommand)]
    subcommand: CacheGammaEventsSubcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum CacheGammaEventsSubcommand {
    ListDateCascades(CacheGammaEventsListDateCascadesCommand),
}

impl CacheGammaEventsCommand {
    pub async fn run(self) -> Result<ExitCode, CacheGammaEventsCommandRunError> {
        use CacheGammaEventsCommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            ListDateCascades(command) => map_err!(command.run().await, CacheGammaEventsListDateCascadesCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsCommandRunError {
    #[error("failed to run cache gamma events list date cascades command")]
    CacheGammaEventsListDateCascadesCommandRunFailed { source: CacheGammaEventsListDateCascadesCommandRunError },
}

mod cache_gamma_events_list_date_cascades_command;

pub use cache_gamma_events_list_date_cascades_command::*;
