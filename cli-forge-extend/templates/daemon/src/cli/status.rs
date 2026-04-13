//! Status daemon command

use clap::Parser;
use std::process::ExitCode;

/// Status command arguments
#[derive(Debug, Clone, Parser)]
pub struct StatusArgs {
    /// Instance ID to query (auto-detect if omitted)
    #[arg(short, long)]
    pub instance_id: Option<String>,

    /// Output format (json or text)
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Status command implementation
pub struct StatusCommand {
    args: StatusArgs,
}

impl StatusCommand {
    pub fn new(args: StatusArgs) -> Self {
        Self { args }
    }

    pub fn execute(&self) -> ExitCode {
        if self.args.verbose {
            eprintln!("Querying daemon status{}...",
                self.args.instance_id
                    .as_ref()
                    .map(|id| format!(" (instance: {})", id))
                    .unwrap_or_default()
            );
        }

        // TODO: Implement actual daemon status query logic
        // - Detect running daemon instance
        // - Send JSON-RPC status request via IPC
        // - Parse and display response

        eprintln!("Daemon status not yet implemented (requires IPC wiring)");
        ExitCode::from(1)
    }
}
