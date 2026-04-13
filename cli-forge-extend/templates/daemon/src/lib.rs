//! Daemon template with JSON-RPC app server protocol
//!
//! This module provides a daemon implementation with support for:
//! - Stdio transport (default for subprocess integration)
//! - WebSocket transport (for remote client connections)
//! - TCP Socket and Unix Domain Socket IPC
//! - JSON-RPC 2.0 protocol

pub mod lifecycle;
pub mod daemon;
pub mod session;
pub mod protocol;
pub mod auth;
pub mod ipc;
pub mod transport;
pub mod cli;

pub use lifecycle::{LifecycleState, HealthStatus};
pub use daemon::DaemonInstance;
pub use session::{ProtocolSession, ClientInfo};
pub use protocol::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, ErrorObject, JsonRpcHandler, DaemonMethodHandler};
pub use auth::{Authenticator, Credentials, AuthMode, AuthError, NoAuthenticator, CapabilityTokenAuthenticator, SignedBearerTokenAuthenticator, JwtClaims, JwtValidator};
pub use transport::{TransportBinding, Transport};
