use Subcommand::*;
use errgonomic::map_err;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, propagate_version = true)]
pub struct Command {
    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum Subcommand {
    Cache(CacheCommand),
}

impl Command {
    pub async fn run(self) -> Result<(), CommandRunError> {
        use CommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            Cache(command) => map_err!(command.run().await, CacheCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CommandRunError {
    #[error("failed to run cache command")]
    CacheCommandRunFailed { source: CacheCommandRunError },
}

mod cache_command;
pub use cache_command::*;
mod cache_download_command;
pub use cache_download_command::*;
mod cache_market_responses_command;
pub use cache_market_responses_command::*;
mod cache_order_book_summary_responses_command;
pub use cache_order_book_summary_responses_command::*;
