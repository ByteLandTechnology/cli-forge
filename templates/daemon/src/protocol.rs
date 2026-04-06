//! JSON-RPC 2.0 protocol implementation

use crate::daemon::DaemonInstance;
use crate::session::ProtocolSession;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// JSON-RPC message variants
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// Request message
    Request(JsonRpcRequest),
    /// Response message
    Response(JsonRpcResponse),
    /// Notification message (no id)
    Notification(JsonRpcNotification),
}

impl JsonRpcMessage {
    /// Parse a JSON-RPC message from JSON string
    pub fn parse(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// JSON-RPC request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcRequest {
    /// Request id
    pub id: Value,
    /// Method name
    pub method: String,
    /// Method parameters
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse {
    /// Success response
    Success {
        /// Response id (matches request id)
        id: Value,
        /// Result object
        result: Value,
    },
    /// Error response
    Error {
        /// Response id (matches request id)
        id: Value,
        /// Error object
        error: ErrorObject,
    },
}

/// JSON-RPC notification (no response expected)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcNotification {
    /// Method name
    pub method: String,
    /// Method parameters
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC error object
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorObject {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional error data
    #[serde(default)]
    pub data: Option<Value>,
}

impl ErrorObject {
    /// Create a new error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create with data
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }
}

/// Standard JSON-RPC error codes
pub mod error_codes {
    /// Server overloaded - retry later
    pub const SERVER_OVERLOADED: i32 = -32001;
    /// Invalid request
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Internal error
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// Create a success response
pub fn success_response(id: Value, result: Value) -> JsonRpcResponse {
    JsonRpcResponse::Success { id, result }
}

/// Create an error response
pub fn error_response(id: Value, error: ErrorObject) -> JsonRpcResponse {
    JsonRpcResponse::Error { id, error }
}

/// Create a success notification
pub fn notification(method: impl Into<String>, params: Value) -> JsonRpcNotification {
    JsonRpcNotification {
        method: method.into(),
        params,
    }
}

/// JSON-RPC handler trait for method dispatch
#[async_trait]
pub trait JsonRpcHandler: Send + Sync {
    /// Handle a JSON-RPC request
    async fn handle(&self, request: JsonRpcRequest, session: Arc<RwLock<ProtocolSession>>) -> JsonRpcResponse;

    /// Handle a JSON-RPC notification (no response)
    async fn handle_notification(&self, notification: JsonRpcNotification, session: Arc<RwLock<ProtocolSession>>);
}

/// Daemon method handler implementing JsonRpcHandler
pub struct DaemonMethodHandler {
    daemon: Arc<RwLock<DaemonInstance>>,
}

impl DaemonMethodHandler {
    /// Create a new handler
    pub fn new(daemon: Arc<RwLock<DaemonInstance>>) -> Self {
        Self { daemon }
    }
}

#[async_trait]
impl JsonRpcHandler for DaemonMethodHandler {
    async fn handle(&self, request: JsonRpcRequest, _session: Arc<RwLock<ProtocolSession>>) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id, request.params).await,
            "ping" => self.handle_ping(request.id).await,
            "start" => self.handle_start(request.id).await,
            "stop" => self.handle_stop(request.id).await,
            "restart" => self.handle_restart(request.id).await,
            "status" => self.handle_status(request.id).await,
            _ => error_response(
                request.id,
                ErrorObject::new(error_codes::METHOD_NOT_FOUND, "Method not found"),
            ),
        }
    }

    async fn handle_notification(&self, notification: JsonRpcNotification, _session: Arc<RwLock<ProtocolSession>>) {
        match notification.method.as_str() {
            "ping" | "status" => {
                // Notifications are acknowledged but not responded to
            }
            _ => {
                // Unknown notification - ignore
            }
        }
    }
}

impl DaemonMethodHandler {
    async fn handle_initialize(&self, id: Value, params: Value) -> JsonRpcResponse {
        // Validate version if provided
        if let Some(version) = params.get("version").and_then(|v| v.as_str()) {
            if version.is_empty() {
                return error_response(
                    id,
                    ErrorObject::new(error_codes::INVALID_REQUEST, "Version cannot be empty"),
                );
            }
        }
        success_response(id, serde_json::json!({
            "version": "1.0.0",
            "protocol": "json-rpc",
            "transport": "daemon"
        }))
    }

    async fn handle_ping(&self, id: Value) -> JsonRpcResponse {
        let daemon = self.daemon.read().await;
        let state = daemon.state();
        let health = daemon.health();
        success_response(id, serde_json::json!({
            "state": state.to_string(),
            "health": health.to_string()
        }))
    }

    async fn handle_start(&self, id: Value) -> JsonRpcResponse {
        let mut daemon = self.daemon.write().await;
        if let Err(e) = daemon.transition_to(crate::lifecycle::LifecycleState::Running) {
            return error_response(
                id,
                ErrorObject::new(error_codes::INTERNAL_ERROR, e.to_string()),
            );
        }
        daemon.start();
        success_response(id, serde_json::json!({
            "state": daemon.state().to_string()
        }))
    }

    async fn handle_stop(&self, id: Value) -> JsonRpcResponse {
        let mut daemon = self.daemon.write().await;
        daemon.stop();
        success_response(id, serde_json::json!({
            "state": daemon.state().to_string()
        }))
    }

    async fn handle_restart(&self, id: Value) -> JsonRpcResponse {
        let mut daemon = self.daemon.write().await;
        daemon.restart();
        success_response(id, serde_json::json!({
            "state": daemon.state().to_string()
        }))
    }

    async fn handle_status(&self, id: Value) -> JsonRpcResponse {
        let daemon = self.daemon.read().await;
        success_response(id, daemon.status())
    }
}

/// Create stateChanged notification
pub fn state_changed_notification(state: &str, health: &str) -> JsonRpcNotification {
    notification("stateChanged", serde_json::json!({ "state": state, "health": health }))
}

/// Create healthChanged notification
pub fn health_changed_notification(health: &str, reason: Option<&str>) -> JsonRpcNotification {
    let mut params = serde_json::json!({ "health": health });
    if let Some(r) = reason {
        params["reason"] = serde_json::json!(r);
    }
    notification("healthChanged", params)
}

/// Create error notification
pub fn error_notification(code: i32, message: &str) -> JsonRpcNotification {
    notification("error", serde_json::json!({ "code": code, "message": message }))
}
