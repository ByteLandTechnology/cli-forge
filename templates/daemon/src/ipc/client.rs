//! IPC client for CLI-to-daemon communication

use crate::protocol::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse};
use crate::transport::TransportBinding;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    #[error("Timeout")]
    Timeout,
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type IpcResult<T> = Result<T, IpcError>;

/// IPC client for communicating with daemon
pub struct IpcClient {
    transport: TransportBinding,
}

impl IpcClient {
    /// Create a new IPC client
    pub fn new(transport: TransportBinding) -> Self {
        Self { transport }
    }

    /// Send a JSON-RPC request and wait for response
    pub async fn send(&self, method: &str, params: Value) -> IpcResult<Value> {
        // For now, this is a placeholder implementation
        // In actual implementation, this would:
        // 1. Connect to the daemon via TCP or Unix socket
        // 2. Send JSON-RPC request
        // 3. Wait for and parse response
        unimplemented!("IPC client implementation depends on transport layer wiring")
    }

    /// Create a JSON-RPC request
    pub fn create_request(id: &str, method: &str, params: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            id: Value::String(id.to_string()),
            method: method.to_string(),
            params,
        }
    }
}
