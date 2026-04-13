//! Daemon lifecycle state and health status

/// Daemon lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LifecycleState {
    /// Daemon not running
    Stopped,
    /// Start in progress
    Starting,
    /// Daemon active and ready
    Running,
    /// Stop in progress
    Stopping,
    /// Startup/exit failure occurred
    Failed,
}

impl LifecycleState {
    /// Returns true if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, LifecycleState::Stopped | LifecycleState::Failed)
    }

    /// Returns true if transition to target state is valid
    pub fn can_transition_to(&self, target: &LifecycleState) -> bool {
        match (self, target) {
            (LifecycleState::Stopped, LifecycleState::Starting) => true,
            (LifecycleState::Starting, LifecycleState::Running) => true,
            (LifecycleState::Starting, LifecycleState::Failed) => true,
            (LifecycleState::Running, LifecycleState::Stopping) => true,
            (LifecycleState::Running, LifecycleState::Failed) => true,
            (LifecycleState::Stopping, LifecycleState::Stopped) => true,
            (LifecycleState::Stopping, LifecycleState::Failed) => true,
            (LifecycleState::Failed, LifecycleState::Starting) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleState::Stopped => write!(f, "stopped"),
            LifecycleState::Starting => write!(f, "starting"),
            LifecycleState::Running => write!(f, "running"),
            LifecycleState::Stopping => write!(f, "stopping"),
            LifecycleState::Failed => write!(f, "failed"),
        }
    }
}

/// Daemon health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Daemon can accept and process requests
    Ready,
    /// Daemon still starting up
    Initializing,
    /// Daemon cannot process requests
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Ready => write!(f, "ready"),
            HealthStatus::Initializing => write!(f, "initializing"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}
