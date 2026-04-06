//! CLI command module

pub mod start;
pub mod status;
pub mod stop;
pub mod restart;
pub mod run;
pub mod schema;

pub use start::StartCommand;
pub use status::StatusCommand;
pub use stop::StopCommand;
pub use restart::RestartCommand;
pub use run::RunCommand;
pub use schema::SchemaCommand;

use crate::daemon::DaemonInstance;
use clap::{Parser, Subcommand};
use std::process::ExitCode;

/// Daemon CLI arguments
#[derive(Debug, Parser)]
#[command(name = "daemon")]
#[command(about = "Daemon process manager")]
pub struct DaemonCli {
    #[command(subcommand)]
    pub command: DaemonCommand,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Daemon subcommands
#[derive(Debug, Subcommand)]
pub enum DaemonCommand {
    /// Start the daemon (TCP or Unix socket mode)
    Start(start::StartArgs),
    /// Run daemon in stdio mode (subprocess integration)
    Run(run::RunArgs),
    /// Stop the daemon
    Stop(stop::StopArgs),
    /// Restart the daemon
    Restart(restart::RestartArgs),
    /// Get daemon status
    Status(status::StatusArgs),
    /// Generate JSON Schema for the protocol
    Schema(schema::SchemaArgs),
}

impl DaemonCli {
    /// Execute the CLI
    pub fn execute(self) -> ExitCode {
        match &self.command {
            DaemonCommand::Start(args) => {
                let cmd = StartCommand::new(args.clone());
                cmd.execute()
            }
            DaemonCommand::Run(args) => {
                let cmd = RunCommand::new(args.clone());
                cmd.execute()
            }
            DaemonCommand::Stop(args) => {
                let cmd = StopCommand::new(args.clone());
                cmd.execute()
            }
            DaemonCommand::Restart(args) => {
                let cmd = RestartCommand::new(args.clone());
                cmd.execute()
            }
            DaemonCommand::Status(args) => {
                let cmd = StatusCommand::new(args.clone());
                cmd.execute()
            }
            DaemonCommand::Schema(args) => {
                let cmd = SchemaCommand::new(args.clone());
                cmd.execute()
            }
        }
    }
}
