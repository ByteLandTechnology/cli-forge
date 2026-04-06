//! Protocol session management

use crate::auth::{AuthMode, Credentials};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Client information from initialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientInfo {
    /// Remote address (for tracking)
    pub remote_addr: String,
    /// Authentication mode used
    pub auth_mode: AuthMode,
    /// Credentials used (if any)
    pub credentials: Option<Credentials>,
}

impl ClientInfo {
    /// Create a new client info
    pub fn new(remote_addr: impl Into<String>) -> Self {
        Self {
            remote_addr: remote_addr.into(),
            auth_mode: AuthMode::None,
            credentials: None,
        }
    }

    /// Create with full info
    pub fn with_auth(
        remote_addr: impl Into<String>,
        auth_mode: AuthMode,
        credentials: Option<Credentials>,
    ) -> Self {
        Self {
            remote_addr: remote_addr.into(),
            auth_mode,
            credentials,
        }
    }
}

/// Notification subscription
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Subscription {
    /// Subscribe to state change notifications
    StateChanged,
    /// Subscribe to health change notifications
    HealthChanged,
    /// Subscribe to error notifications
    Error,
}

/// A connected protocol session
#[derive(Debug, Clone)]
pub struct ProtocolSession {
    /// Unique session identifier
    pub id: String,
    /// Client information
    pub client_info: Option<ClientInfo>,
    /// Whether authentication succeeded
    pub authenticated: bool,
    /// Active subscriptions
    pub subscriptions: HashSet<Subscription>,
    /// Connection timestamp
    pub connected_at: std::time::Instant,
}

impl ProtocolSession {
    /// Create a new session
    pub fn new(client_info: ClientInfo) -> Self {
        Self {
            id: uuid_v4(),
            client_info: Some(client_info),
            authenticated: false,
            subscriptions: HashSet::new(),
            connected_at: std::time::Instant::now(),
        }
    }

    /// Create an unauthenticated session
    pub fn new_unauthenticated(remote_addr: impl Into<String>) -> Self {
        Self {
            id: uuid_v4(),
            client_info: Some(ClientInfo::new(remote_addr)),
            authenticated: false,
            subscriptions: HashSet::new(),
            connected_at: std::time::Instant::now(),
        }
    }

    /// Mark session as authenticated
    pub fn set_authenticated(&mut self, auth_mode: AuthMode, credentials: Option<Credentials>) {
        self.authenticated = true;
        if let Some(ref mut info) = self.client_info {
            info.auth_mode = auth_mode;
            info.credentials = credentials;
        }
    }

    /// Subscribe to a notification type
    pub fn subscribe(&mut self, subscription: Subscription) {
        self.subscriptions.insert(subscription);
    }

    /// Unsubscribe from a notification type
    pub fn unsubscribe(&mut self, subscription: &Subscription) {
        self.subscriptions.remove(subscription);
    }

    /// Check if subscribed to a notification type
    pub fn is_subscribed(&self, subscription: &Subscription) -> bool {
        self.subscriptions.contains(subscription)
    }

    /// Get session duration
    pub fn duration(&self) -> std::time::Duration {
        self.connected_at.elapsed()
    }
}

/// Generate a simple UUID (placeholder - use uuid crate in production)
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("session-{:x}", now)
}
