//! Daemon core implementation

use crate::lifecycle::{HealthStatus, LifecycleState};
use crate::protocol::{self, JsonRpcMessage};
use crate::session::{ClientInfo, ProtocolSession};
use crate::transport::{Transport, TransportBinding};
use crate::ErrorObject;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Daemon instance
pub struct DaemonInstance {
    /// Unique instance identifier
    instance_id: String,
    /// Current lifecycle state
    state: LifecycleState,
    /// Current health status
    health: HealthStatus,
    /// Transport binding
    transport: TransportBinding,
    /// Failure reason (if failed)
    failure_reason: Option<String>,
    /// Active sessions
    sessions: HashMap<String, ProtocolSession>,
    /// Request queue (for overload handling)
    request_queue: Vec<()>,
    /// Maximum concurrent requests
    max_concurrent_requests: usize,
    /// WebSocket server (if running)
    ws_server: Option<Arc<crate::transport::WsServer>>,
}

impl DaemonInstance {
    /// Create a new daemon instance
    pub fn new(transport: TransportBinding) -> Self {
        Self {
            instance_id: uuid_v4(),
            state: LifecycleState::Stopped,
            health: HealthStatus::Initializing,
            transport,
            failure_reason: None,
            sessions: HashMap::new(),
            request_queue: Vec::new(),
            max_concurrent_requests: 100,
            ws_server: None,
        }
    }

    /// Get the instance ID
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get current lifecycle state
    pub fn state(&self) -> LifecycleState {
        self.state
    }

    /// Get current health status
    pub fn health(&self) -> HealthStatus {
        self.health
    }

    /// Get the transport binding
    pub fn transport(&self) -> &TransportBinding {
        &self.transport
    }

    /// Get failure reason
    pub fn failure_reason(&self) -> Option<&str> {
        self.failure_reason.as_deref()
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: LifecycleState) -> Result<(), InvalidTransition> {
        if !self.state.can_transition_to(&new_state) {
            return Err(InvalidTransition(self.state, new_state));
        }
        self.state = new_state;
        Ok(())
    }

    /// Start the daemon
    pub fn start(&mut self) {
        if self.state == LifecycleState::Stopped || self.state == LifecycleState::Failed {
            self.state = LifecycleState::Starting;
            self.health = HealthStatus::Initializing;
            self.failure_reason = None;
            // Simulate async start - in real impl this would be async
            self.state = LifecycleState::Running;
            self.health = HealthStatus::Ready;
        }
    }

    /// Stop the daemon
    pub fn stop(&mut self) {
        if self.state == LifecycleState::Running {
            self.state = LifecycleState::Stopping;
            // Simulate async stop - in real impl this would be async
            self.state = LifecycleState::Stopped;
            self.health = HealthStatus::Unhealthy;
        }
    }

    /// Restart the daemon
    pub fn restart(&mut self) {
        let was_running = self.state == LifecycleState::Running;
        if was_running {
            self.state = LifecycleState::Stopping;
        }
        self.state = LifecycleState::Starting;
        self.health = HealthStatus::Initializing;
        // Simulate async restart - in real impl this would be async
        self.state = LifecycleState::Running;
        self.health = HealthStatus::Ready;
    }

    /// Get daemon status as JSON value
    pub fn status(&self) -> serde_json::Value {
        serde_json::json!({
            "lifecycleState": self.state.to_string(),
            "health": self.health.to_string(),
            "instanceId": self.instance_id,
            "reason": self.failure_reason,
            "nextAction": self.recommended_action(),
        })
    }

    /// Get recommended next action
    fn recommended_action(&self) -> Option<&str> {
        match self.state {
            LifecycleState::Failed => Some("restart"),
            LifecycleState::Stopped => Some("start"),
            _ => None,
        }
    }

    /// Register a new session
    pub fn register_session(&mut self, session: ProtocolSession) {
        self.sessions.insert(session.id.clone(), session);
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    /// Check if request queue has capacity
    pub fn has_queue_capacity(&self) -> bool {
        self.request_queue.len() < self.max_concurrent_requests
    }

    /// Get overload error if queue is full
    pub fn check_overload(&self) -> Option<ErrorObject> {
        if self.has_queue_capacity() {
            None
        } else {
            Some(ErrorObject::new(
                protocol::error_codes::SERVER_OVERLOADED,
                "Server overloaded; retry later",
            ))
        }
    }

    /// Run daemon with stdio transport (blocking loop)
    pub async fn run_stdio(daemon: Arc<RwLock<DaemonInstance>>) -> Result<(), std::io::Error> {
        use crate::transport::StdioTransport;
        use crate::protocol::JsonRpcHandler;

        let transport = StdioTransport::new();
        let handler = crate::protocol::DaemonMethodHandler::new(daemon.clone());

        loop {
            // Receive message
            let msg = transport.recv().await?;
            match msg {
                crate::protocol::JsonRpcMessage::Request(req) => {
                    let session = Arc::new(RwLock::new(ProtocolSession::new(crate::session::ClientInfo {
                        remote_addr: "stdio".to_string(),
                        auth_mode: crate::auth::AuthMode::None,
                        credentials: None,
                    })));
                    let response = handler.handle(req, session).await;
                    // Send response
                    transport.send(crate::protocol::JsonRpcMessage::Response(response)).await?;
                }
                crate::protocol::JsonRpcMessage::Notification(notif) => {
                    let session = Arc::new(RwLock::new(ProtocolSession::new(crate::session::ClientInfo {
                        remote_addr: "stdio".to_string(),
                        auth_mode: crate::auth::AuthMode::None,
                        credentials: None,
                    })));
                    handler.handle_notification(notif, session).await;
                }
                crate::protocol::JsonRpcMessage::Response(_) => {
                    // Ignore unexpected responses in stdio mode
                }
            }
        }
    }

    /// Run daemon with WebSocket transport
    pub async fn run_websocket(
        daemon: Arc<RwLock<DaemonInstance>>,
        addr: std::net::SocketAddr,
        authenticator: Option<Arc<dyn crate::auth::Authenticator>>,
    ) -> Result<(), std::io::Error> {
        use crate::transport::{WsServer, WsServerConfig};

        let config = WsServerConfig::new();
        let server = WsServer::new(config, authenticator);

        // Store server reference
        {
            let mut daemon_guard = daemon.write().await;
            daemon_guard.ws_server = Some(Arc::new(server));
        }

        // Get server reference and start serving
        let daemon_guard = daemon.read().await;
        if let Some(ref ws_server) = daemon_guard.ws_server {
            ws_server.serve(addr).await?;
        }

        Ok(())
    }
}

impl Clone for DaemonInstance {
    fn clone(&self) -> Self {
        Self {
            instance_id: self.instance_id.clone(),
            state: self.state.clone(),
            health: self.health.clone(),
            transport: self.transport.clone(),
            failure_reason: self.failure_reason.clone(),
            sessions: HashMap::new(),
            request_queue: Vec::new(),
            max_concurrent_requests: self.max_concurrent_requests,
            ws_server: None, // Don't clone server - new instance
        }
    }
}

/// Invalid state transition error
#[derive(Debug)]
pub struct InvalidTransition(LifecycleState, LifecycleState);

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid transition from {} to {}",
            self.0, self.1
        )
    }
}

impl std::error::Error for InvalidTransition {}

/// Generate a simple UUID (placeholder - use uuid crate in production)
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("daemon-{:x}", now)
}
