//! Stop daemon command

use clap::Parser;
use std::process::ExitCode;

/// Stop command arguments
#[derive(Debug, Clone, Parser)]
pub struct StopArgs {
    /// Instance ID to stop (auto-detect if omitted)
    #[arg(short, long)]
    pub instance_id: Option<String>,

    /// Force stop (do not wait for graceful shutdown)
    #[arg(short, long)]
    pub force: bool,

    /// Timeout in seconds for graceful shutdown
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Stop command implementation
pub struct StopCommand {
    args: StopArgs,
}

impl StopCommand {
    pub fn new(args: StopArgs) -> Self {
        Self { args }
    }

    pub fn execute(&self) -> ExitCode {
        if self.args.verbose {
            eprintln!("Stopping daemon{}...",
                self.args.instance_id
                    .as_ref()
                    .map(|id| format!(" (instance: {})", id))
                    .unwrap_or_default()
            );
        }

        // TODO: Implement actual daemon stop logic
        // - Detect running daemon instance
        // - Send JSON-RPC stop request via IPC
        // - Wait for confirmation or timeout
        // - Force kill if --force

        eprintln!("Daemon stop not yet implemented (requires IPC wiring)");
        ExitCode::from(1)
    }
}
