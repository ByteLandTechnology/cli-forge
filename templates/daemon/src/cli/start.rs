//! Start daemon command

use clap::Parser;
use std::process::ExitCode;

/// Start command arguments
#[derive(Debug, Clone, Parser)]
pub struct StartArgs {
    /// Transport type: stdio, tcp, unix
    #[arg(long, default_value = "tcp")]
    pub transport: String,

    /// Transport binding host (for TCP mode)
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port number (for TCP mode)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Unix socket path (mutually exclusive with --port)
    #[arg(long)]
    pub socket: Option<String>,

    /// Authentication mode: capability-token, signed-bearer-token
    #[arg(long)]
    pub ws_auth: Option<String>,

    /// Token file for capability token auth
    #[arg(long)]
    pub ws_token_file: Option<String>,

    /// Shared secret file for JWT auth
    #[arg(long)]
    pub ws_shared_secret_file: Option<String>,

    /// Expected JWT issuer
    #[arg(long)]
    pub ws_issuer: Option<String>,

    /// Expected JWT audience
    #[arg(long)]
    pub ws_audience: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Start command implementation
pub struct StartCommand {
    args: StartArgs,
}

impl StartCommand {
    pub fn new(args: StartArgs) -> Self {
        Self { args }
    }

    pub fn execute(&self) -> ExitCode {
        if self.args.verbose {
            eprintln!("Starting daemon...");
        }

        let transport = TransportBinding::from_args(&self.args);

        if self.args.verbose {
            eprintln!("Transport: {:?}", transport);
            if let Some(ref auth) = self.args.ws_auth {
                eprintln!("Auth mode: {}", auth);
            }
        }

        // TODO: Implement actual daemon start logic
        // - Check if daemon already running
        // - Spawn daemon process
        // - Wait for ready signal
        // - Report instance ID

        eprintln!("Daemon start not yet implemented (requires IPC wiring)");
        ExitCode::from(1)
    }
}

/// Transport binding from CLI args
pub struct TransportBinding;

impl TransportBinding {
    /// Create from start arguments
    pub fn from_args(args: &StartArgs) -> crate::transport::TransportBinding {
        match args.transport.as_str() {
            "stdio" => crate::transport::TransportBinding::Stdio,
            "unix" => crate::transport::TransportBinding::UnixSocket {
                path: args.socket.clone().unwrap_or_else(|| "/tmp/daemon.sock".to_string()),
            },
            "tcp" | _ => crate::transport::TransportBinding::TcpSocket {
                host: args.host.clone(),
                port: args.port.unwrap_or(9090),
            },
        }
    }
}
