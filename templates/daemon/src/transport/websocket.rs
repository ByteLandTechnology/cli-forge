//! WebSocket transport for daemon mode
//!
//! This module provides WebSocket transport support for the daemon.
//! Note: Full WebSocket server implementation is pending runtime integration.

use crate::auth::{Authenticator, AuthMode, Credentials};
use crate::protocol::JsonRpcMessage;
use async_trait::async_trait;
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// TLS configuration for secure WebSocket (wss://)
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to TLS certificate file
    cert_file: String,
    /// Path to TLS private key file
    key_file: String,
}

impl TlsConfig {
    /// Create TLS configuration from certificate and key files
    pub fn new(cert_file: impl Into<String>, key_file: impl Into<String>) -> Self {
        Self {
            cert_file: cert_file.into(),
            key_file: key_file.into(),
        }
    }

    /// Load TLS config from files
    pub fn from_files(cert_path: &Path, key_path: &Path) -> io::Result<Self> {
        if !cert_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Certificate file not found: {}", cert_path.display()),
            ));
        }
        if !key_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Key file not found: {}", key_path.display()),
            ));
        }
        Ok(Self::new(
            cert_path.to_string_lossy().into_owned(),
            key_path.to_string_lossy().into_owned(),
        ))
    }

    /// Get certificate file path
    pub fn cert_file(&self) -> &str {
        &self.cert_file
    }

    /// Get key file path
    pub fn key_file(&self) -> &str {
        &self.key_file
    }
}

/// WebSocket server configuration
#[derive(Debug, Clone)]
pub struct WsServerConfig {
    /// TLS configuration (if using wss://)
    tls_config: Option<TlsConfig>,
    /// Maximum concurrent connections
    max_connections: usize,
    /// Connection timeout in seconds
    connection_timeout_secs: u64,
}

impl WsServerConfig {
    /// Create new server config
    pub fn new() -> Self {
        Self {
            tls_config: None,
            max_connections: 100,
            connection_timeout_secs: 30,
        }
    }

    /// Set TLS configuration
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls_config = Some(tls);
        self
    }

    /// Set maximum connections
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Check if TLS is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.tls_config.is_some()
    }

    /// Get TLS config
    pub fn tls_config(&self) -> Option<&TlsConfig> {
        self.tls_config.as_ref()
    }
}

impl Default for WsServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket message wrapper
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Vec<u8>),
    /// Close frame
    Close,
}

impl WebSocketMessage {
    /// Parse as JSON-RPC message
    pub fn to_jsonrpc(self) -> io::Result<JsonRpcMessage> {
        match self {
            WebSocketMessage::Text(s) => serde_json::from_str(&s)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            WebSocketMessage::Binary(b) => serde_json::from_slice(&b)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            WebSocketMessage::Close => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Received close frame",
            )),
        }
    }

    /// Create from JSON-RPC message
    pub fn from_jsonrpc(msg: &JsonRpcMessage) -> io::Result<Self> {
        let s = serde_json::to_string(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(WebSocketMessage::Text(s))
    }
}

/// WebSocket connection state
#[derive(Debug, Clone)]
pub enum ConnectionState {
    /// Handshake not yet complete
    Handshake,
    /// Awaiting authentication
    Authenticating,
    /// Authenticated and ready
    Authenticated,
    /// Connection closed
    Closed,
}

/// WebSocket transport for daemon RPC
pub struct WebSocketTransport {
    /// Unique connection ID
    conn_id: String,
    /// Current connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Authenticator
    authenticator: Option<Arc<dyn Authenticator>>,
    /// Remote peer address
    peer_addr: String,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport
    pub fn new(
        conn_id: String,
        authenticator: Option<Arc<dyn Authenticator>>,
        peer_addr: String,
    ) -> Self {
        Self {
            conn_id,
            state: Arc::new(RwLock::new(ConnectionState::Handshake)),
            authenticator,
            peer_addr,
        }
    }

    /// Check if authentication is required
    pub fn requires_auth(&self) -> bool {
        self.authenticator
            .as_ref()
            .map(|a| a.mode() != AuthMode::None)
            .unwrap_or(false)
    }

    /// Authenticate credentials
    pub async fn authenticate(&self, credentials: &Credentials) -> io::Result<()> {
        match &self.authenticator {
            Some(auth) => auth
                .authenticate(credentials)
                .map_err(|e| io::Error::new(io::ErrorKind::PermissionDenied, e)),
            None => Ok(()),
        }
    }

    /// Transition to authenticated state
    pub async fn set_authenticated(&self) {
        let mut state = self.state.write().await;
        *state = ConnectionState::Authenticated;
    }

    /// Get connection ID
    pub fn conn_id(&self) -> &str {
        &self.conn_id
    }
}

#[async_trait]
impl super::Transport for WebSocketTransport {
    async fn send(&self, _message: JsonRpcMessage) -> io::Result<()> {
        // WebSocket send requires runtime integration
        // This is a stub - actual implementation needs tokio-tungstenite
        tracing::debug!("WebSocket send (stub)");
        Ok(())
    }

    async fn recv(&self) -> io::Result<JsonRpcMessage> {
        // WebSocket recv requires runtime integration
        // This is a stub - actual implementation needs tokio-tungstenite
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "WebSocket recv not implemented - needs runtime integration",
        ))
    }

    fn is_connected(&self) -> bool {
        true
    }

    fn peer_info(&self) -> String {
        format!("ws:{}", self.peer_addr)
    }
}

/// WebSocket server for handling incoming connections
///
/// Note: Full implementation requires tokio-tungstenite integration.
/// This is a stub that logs but doesn't actually accept connections.
pub struct WsServer {
    /// Server configuration
    config: WsServerConfig,
    /// Authenticator (if required)
    authenticator: Option<Arc<dyn Authenticator>>,
}

impl WsServer {
    /// Create a new WebSocket server
    pub fn new(config: WsServerConfig, authenticator: Option<Arc<dyn Authenticator>>) -> Self {
        Self {
            config,
            authenticator,
        }
    }

    /// Start the WebSocket server
    ///
    /// Note: This is a stub implementation.
    /// Full implementation requires tokio-tungstenite with proper async WebSocket handling.
    pub async fn serve(&self, _addr: SocketAddr) -> io::Result<()> {
        tracing::info!(
            "WebSocket server stub started on {:?} (TLS: {})",
            _addr,
            self.config.is_tls_enabled()
        );
        // TODO: Implement full WebSocket server with tokio-tungstenite
        // - Use tokio::net::TcpListener to accept connections
        // - Use tokio_tungstenite::accept_async for WebSocket upgrade
        // - Handle multiple concurrent connections
        // - Implement JSON-RPC message framing
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "WebSocket server not implemented - requires tokio-tungstenite integration",
        ))
    }

    /// Get connection count (stub)
    pub async fn connection_count(&self) -> usize {
        0
    }

    /// Broadcast to all connections (stub)
    pub async fn broadcast(&self, _msg: JsonRpcMessage) -> io::Result<()> {
        tracing::debug!("WebSocket broadcast (stub)");
        Ok(())
    }
}

impl std::fmt::Debug for WsServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsServer")
            .field("config", &self.config)
            .field("authenticator", &self.authenticator.is_some())
            .finish()
    }
}
