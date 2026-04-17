//! Notifications plugin for Gateorix.
//!
//! Sends desktop notifications through the host OS notification system.
//! Requires "notifications" permission in the app manifest.

use gateorix_host_core::ipc::protocol::{IpcRequest, IpcResponse};
use gateorix_host_core::plugins::Plugin;
use tracing::info;

pub struct NotificationsPlugin;

impl NotificationsPlugin {
    pub fn new() -> Self {
        Self
    }

    fn send_notification(&self, request: &IpcRequest) -> IpcResponse {
        let title = request
            .payload
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Gateorix");

        let body = match request.payload.get("body").and_then(|b| b.as_str()) {
            Some(b) => b,
            None => return IpcResponse::error(&request.id, "missing 'body' in payload"),
        };

        // Placeholder: actual implementation will use a platform notification crate
        // (e.g. notify-rust on Linux, native APIs on macOS/Windows)
        info!(title = %title, body = %body, "notification sent (stub)");

        IpcResponse::ok(&request.id, serde_json::json!({ "sent": true }))
    }
}

impl Default for NotificationsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for NotificationsPlugin {
    fn namespace(&self) -> &str {
        "notifications"
    }

    fn handle(&self, request: &IpcRequest) -> IpcResponse {
        let action = request.channel.strip_prefix("notifications.").unwrap_or("");
        match action {
            "send" => self.send_notification(request),
            _ => IpcResponse::error(
                &request.id,
                format!("unknown notifications action: {}", action),
            ),
        }
    }

    fn on_init(&self) {
        info!("notifications plugin initialized");
    }
}
