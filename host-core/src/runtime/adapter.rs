//! Adapter trait and protocol definitions.
//!
//! All runtime adapters (Python, Go, .NET, Swift, etc.) conform to
//! a common protocol so the host core can launch, communicate with,
//! and tear down backends uniformly.

use serde::{Deserialize, Serialize};

/// Transport method between host core and the sidecar process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    /// Communication over the process's stdin/stdout.
    Stdio,
    /// Communication over a local HTTP endpoint.
    Http,
}

/// Configuration needed to launch a runtime adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Adapter type identifier (e.g. "python", "go", "dotnet").
    pub runtime_type: String,
    /// Path to the entry point (e.g. "backend/main.py").
    pub entry: String,
    /// Preferred transport method.
    #[serde(default = "default_transport")]
    pub transport: TransportKind,
    /// Optional port for HTTP transport.
    pub port: Option<u16>,
    /// Optional environment variables passed to the sidecar.
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

fn default_transport() -> TransportKind {
    TransportKind::Stdio
}

/// Status of a running adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterStatus {
    Starting,
    Running,
    Stopped,
    Crashed(String),
}

/// Trait that all runtime adapter implementations must satisfy.
///
/// The host core interacts with adapters exclusively through this trait,
/// making the backend language transparent.
pub trait RuntimeAdapter: Send + Sync {
    /// Start the sidecar process.
    fn start(&mut self) -> Result<(), AdapterError>;

    /// Stop the sidecar process gracefully.
    fn stop(&mut self) -> Result<(), AdapterError>;

    /// Send a JSON message to the sidecar and receive a response.
    fn send(&self, message: &str) -> Result<String, AdapterError>;

    /// Get the current status of the adapter.
    fn status(&self) -> AdapterStatus;

    /// The runtime type identifier.
    fn runtime_type(&self) -> &str;
}

/// Errors from adapter operations.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("failed to start adapter: {0}")]
    StartFailed(String),

    #[error("adapter not running")]
    NotRunning,

    #[error("communication error: {0}")]
    Communication(String),

    #[error("adapter process exited unexpectedly: {0}")]
    ProcessExited(String),
}
