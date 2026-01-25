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
    CacheDownload(CacheDownloadCommand),
}

impl Command {
    pub async fn run(self) -> Result<(), CommandRunError> {
        use CommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            CacheDownload(command) => map_err!(command.run().await, CacheDownloadCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum CommandRunError {
    #[error("failed to run cache download command")]
    CacheDownloadCommandRunFailed { source: CacheDownloadCommandRunError },
}

mod cache_download_command;
pub use cache_download_command::*;
