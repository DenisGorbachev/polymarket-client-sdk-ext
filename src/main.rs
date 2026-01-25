use clap::Parser;
use errgonomic::exit_result;
use polymarket_client_sdk_ext::Command;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Command::parse();
    let result = args.run().await;
    exit_result(result)
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Command::command().debug_assert();
}
