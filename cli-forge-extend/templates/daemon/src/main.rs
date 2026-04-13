//! Daemon binary entry point

use daemon_template::cli::DaemonCli;
use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = DaemonCli::parse();
    let code = cli.execute();
    // Match on ExitCode to get the inner i32 value
    // ExitCode is either ExitCode::SUCCESS (0) or ExitCode::FAILURE (1) internally
    #[allow(deprecated)]
    let exit_code = match code {
        std::process::ExitCode::SUCCESS => 0,
        std::process::ExitCode::FAILURE => 1,
        _ => 1,
    };
    std::process::exit(exit_code);
}
