use ClobSubcommand::*;
use errgonomic::map_err;
use std::process::ExitCode;
use thiserror::Error;

#[derive(clap::Parser, Clone, Debug)]
pub struct ClobCommand {
    #[command(subcommand)]
    subcommand: ClobSubcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum ClobSubcommand {
    PlaceLimitOrder(clob_place_limit_order_command::ClobPlaceLimitOrderCommand),
}

impl ClobCommand {
    pub async fn run(self) -> Result<ExitCode, ClobCommandRunError> {
        use ClobCommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            PlaceLimitOrder(command) => map_err!(command.run().await, ClobPlaceLimitOrderCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum ClobCommandRunError {
    #[error("failed to run clob place limit order command")]
    ClobPlaceLimitOrderCommandRunFailed { source: ClobPlaceLimitOrderCommandRunError },
}

mod clob_place_limit_order_command;

pub use clob_place_limit_order_command::*;
