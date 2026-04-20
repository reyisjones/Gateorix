//! IPC message protocol.
//!
//! Defines the JSON message contracts used for communication across
//! the bridge boundary. All messages are validated before dispatch.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An IPC request sent from the frontend to the host core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    /// Unique request identifier for correlating responses.
    pub id: String,
    /// Target channel (e.g. "filesystem.readText", "runtime.invoke").
    pub channel: String,
    /// Arbitrary JSON payload.
    pub payload: serde_json::Value,
}

/// An IPC response returned from the host core to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// The request ID this response corresponds to.
    pub id: String,
    /// Whether the operation succeeded.
    pub ok: bool,
    /// Response payload on success, or error details on failure.
    pub payload: serde_json::Value,
}

/// An event pushed from the host core to the frontend (fire-and-forget).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEvent {
    /// Event name (e.g. "runtime.stdout", "window.close-requested").
    pub event: String,
    /// Event payload.
    pub payload: serde_json::Value,
}

impl IpcRequest {
    /// Generate a new request with a random UUID.
    pub fn new(channel: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            channel: channel.into(),
            payload,
        }
    }
}

impl IpcResponse {
    /// Create a success response.
    pub fn ok(id: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            ok: true,
            payload,
        }
    }

    /// Create an error response.
    pub fn error(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ok: false,
            payload: serde_json::json!({ "error": message.into() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serialization_roundtrip() {
        let req = IpcRequest::new(
            "filesystem.readText",
            serde_json::json!({"path": "./data/notes.txt"}),
        );
        let json = serde_json::to_string(&req).unwrap();
        let parsed: IpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.channel, "filesystem.readText");
        assert_eq!(parsed.id, req.id);
    }

    #[test]
    fn response_ok_and_error() {
        let ok = IpcResponse::ok("req-1", serde_json::json!({"content": "hello"}));
        assert!(ok.ok);

        let err = IpcResponse::error("req-2", "file not found");
        assert!(!err.ok);
    }
}
