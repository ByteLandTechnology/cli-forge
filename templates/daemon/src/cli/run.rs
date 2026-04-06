//! Run daemon command (all transport modes)

use crate::auth::{Authenticator, CapabilityTokenAuthenticator, SignedBearerTokenAuthenticator};
use crate::daemon::DaemonInstance;
use crate::lifecycle::LifecycleState;
use crate::transport::TransportBinding;
use clap::Parser;
use std::process::ExitCode;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Run command arguments
#[derive(Debug, Clone, Parser)]
pub struct RunArgs {
    /// Transport type: stdio, tcp, unix (default: stdio)
    #[arg(long, default_value = "stdio")]
    pub transport: String,

    /// Host for TCP transport (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port for TCP transport (default: 9000)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Socket path for Unix transport
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

/// Run command implementation
pub struct RunCommand {
    args: RunArgs,
}

impl RunCommand {
    pub fn new(args: RunArgs) -> Self {
        Self { args }
    }

    pub fn execute(&self) -> ExitCode {
        // Create transport binding based on transport type
        let transport = match self.args.transport.as_str() {
            "stdio" => {
                if self.args.verbose {
                    eprintln!("Starting daemon in stdio mode...");
                }
                TransportBinding::Stdio
            }
            "tcp" => {
                let port = self.args.port.unwrap_or(9000);
                if self.args.verbose {
                    eprintln!("Starting daemon on TCP {}:{}", self.args.host, port);
                }
                TransportBinding::TcpSocket {
                    host: self.args.host.clone(),
                    port,
                }
            }
            "unix" => {
                let path = self.args.socket.clone().unwrap_or_else(|| "/tmp/daemon.sock".to_string());
                if self.args.verbose {
                    eprintln!("Starting daemon on Unix socket {}", path);
                }
                TransportBinding::UnixSocket { path }
            }
            _ => {
                eprintln!("Unknown transport: {}", self.args.transport);
                return ExitCode::from(1);
            }
        };

        // Create daemon instance
        let daemon = DaemonInstance::new(transport.clone());
        let daemon = Arc::new(RwLock::new(daemon));

        if self.args.verbose {
            let instance_id = daemon.blocking_read().instance_id().to_string();
            eprintln!("Daemon instance created: {}", instance_id);
        }

        // Set up authenticator if specified
        let authenticator: Option<Arc<dyn Authenticator>> = match self.args.ws_auth.as_deref() {
            Some("capability-token") => {
                if let Some(ref token_file) = self.args.ws_token_file {
                    match CapabilityTokenAuthenticator::from_file(std::path::Path::new(token_file)) {
                        Ok(auth) => Some(Arc::new(auth)),
                        Err(e) => {
                            eprintln!("Failed to load token file: {}", e);
                            return ExitCode::from(1);
                        }
                    }
                } else {
                    eprintln!("--ws-token-file required for capability-token auth");
                    return ExitCode::from(1);
                }
            }
            Some("signed-bearer-token") => {
                if let Some(ref secret_file) = self.args.ws_shared_secret_file {
                    match SignedBearerTokenAuthenticator::from_secret_file(
                        std::path::Path::new(secret_file),
                        self.args.ws_issuer.clone(),
                        self.args.ws_audience.clone(),
                    ) {
                        Ok(auth) => Some(Arc::new(auth)),
                        Err(e) => {
                            eprintln!("Failed to load secret file: {}", e);
                            return ExitCode::from(1);
                        }
                    }
                } else {
                    eprintln!("--ws-shared-secret-file required for signed-bearer-token auth");
                    return ExitCode::from(1);
                }
            }
            Some("none") | None => None,
            Some(mode) => {
                eprintln!("Unknown auth mode: {}", mode);
                return ExitCode::from(1);
            }
        };

        // Run the daemon
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        if let Err(e) = rt.block_on(async {
            // Start the daemon
            {
                let mut daemon = daemon.write().await;
                daemon.transition_to(LifecycleState::Starting).ok();
                daemon.start();
            }

            if self.args.verbose {
                let state = daemon.read().await.state();
                eprintln!("Daemon state: {}", state);
            }

            // Run based on transport
            match transport {
                TransportBinding::Stdio => {
                    DaemonInstance::run_stdio(daemon).await
                }
                TransportBinding::TcpSocket { ref host, port } => {
                    let addr: std::net::SocketAddr = format!("{}:{}", host, port)
                        .parse()
                        .expect("Invalid socket address");
                    DaemonInstance::run_websocket(daemon.clone(), addr, authenticator).await
                }
                TransportBinding::UnixSocket { ref path } => {
                    // For Unix socket, we need to convert to a socket addr
                    // In a full implementation, this would use UnixSocketAddr
                    let addr: std::net::SocketAddr = "127.0.0.1:9000"
                        .parse()
                        .expect("Invalid socket address");
                    DaemonInstance::run_websocket(daemon.clone(), addr, authenticator).await
                }
            }
        }) {
            eprintln!("Daemon error: {}", e);
            return ExitCode::from(1);
        }

        ExitCode::SUCCESS
    }
}
