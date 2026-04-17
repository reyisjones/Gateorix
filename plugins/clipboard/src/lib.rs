//! Clipboard plugin for Gateorix.
//!
//! Provides read and write access to the system clipboard.
//! Requires "clipboard" permission in the app manifest.

use gateorix_host_core::ipc::protocol::{IpcRequest, IpcResponse};
use gateorix_host_core::plugins::Plugin;
use tracing::info;

pub struct ClipboardPlugin;

impl ClipboardPlugin {
    pub fn new() -> Self {
        Self
    }

    fn read_text(&self, request: &IpcRequest) -> IpcResponse {
        // Placeholder: actual implementation will use a clipboard crate
        // (e.g. arboard or clipboard-rs)
        info!("clipboard read (stub)");
        IpcResponse::ok(&request.id, serde_json::json!({ "text": "" }))
    }

    fn write_text(&self, request: &IpcRequest) -> IpcResponse {
        let text = match request.payload.get("text").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => return IpcResponse::error(&request.id, "missing 'text' in payload"),
        };

        // Placeholder: actual implementation will use a clipboard crate
        info!(text = %text, "clipboard write (stub)");
        IpcResponse::ok(&request.id, serde_json::json!({ "written": true }))
    }
}

impl Default for ClipboardPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ClipboardPlugin {
    fn namespace(&self) -> &str {
        "clipboard"
    }

    fn handle(&self, request: &IpcRequest) -> IpcResponse {
        let action = request.channel.strip_prefix("clipboard.").unwrap_or("");
        match action {
            "readText" => self.read_text(request),
            "writeText" => self.write_text(request),
            _ => IpcResponse::error(
                &request.id,
                format!("unknown clipboard action: {}", action),
            ),
        }
    }

    fn on_init(&self) {
        info!("clipboard plugin initialized");
    }
}
