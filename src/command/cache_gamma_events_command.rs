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
    MonitorDateCascades(CacheGammaEventsMonitorDateCascadesCommand),
}

impl CacheGammaEventsCommand {
    pub async fn run(self) -> Result<ExitCode, CacheGammaEventsCommandRunError> {
        use CacheGammaEventsCommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            ListDateCascades(command) => map_err!(command.run().await, CacheGammaEventsListDateCascadesCommandRunFailed),
            MonitorDateCascades(command) => map_err!(command.run().await, CacheGammaEventsMonitorDateCascadesCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CacheGammaEventsCommandRunError {
    #[error("failed to run cache gamma events list date cascades command")]
    CacheGammaEventsListDateCascadesCommandRunFailed { source: CacheGammaEventsListDateCascadesCommandRunError },
    #[error("failed to run cache gamma events monitor date cascades command")]
    CacheGammaEventsMonitorDateCascadesCommandRunFailed { source: CacheGammaEventsMonitorDateCascadesCommandRunError },
}

mod cache_gamma_events_list_date_cascades_command;

pub use cache_gamma_events_list_date_cascades_command::*;

mod cache_gamma_events_monitor_date_cascades_command;

pub use cache_gamma_events_monitor_date_cascades_command::*;
