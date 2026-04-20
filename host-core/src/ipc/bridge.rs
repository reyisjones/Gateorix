//! IPC bridge dispatcher.
//!
//! Routes incoming IPC requests to the appropriate handler —
//! either a host-core plugin or a runtime adapter.

use std::collections::HashMap;
use tracing::{debug, warn};
use crate::ipc::protocol::{IpcRequest, IpcResponse};

/// Handler function type for IPC channels.
pub type ChannelHandler = Box<dyn Fn(&IpcRequest) -> IpcResponse + Send + Sync>;

/// The IPC bridge routes frontend requests to registered channel handlers.
pub struct Bridge {
    handlers: HashMap<String, ChannelHandler>,
}

impl Bridge {
    /// Create a new empty bridge.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for the given channel prefix.
    ///
    /// For example, registering "filesystem" will handle channels like
    /// "filesystem.readText", "filesystem.writeText", etc.
    pub fn register_handler(&mut self, channel: impl Into<String>, handler: ChannelHandler) {
        self.handlers.insert(channel.into(), handler);
    }

    /// Dispatch an incoming request to the appropriate handler.
    ///
    /// Channel routing: the bridge splits the channel on '.' and looks up
    /// the first segment as the handler key. If no handler is found,
    /// an error response is returned.
    pub fn dispatch(&self, request: &IpcRequest) -> IpcResponse {
        let namespace = request
            .channel
            .split('.')
            .next()
            .unwrap_or(&request.channel);

        debug!(
            id = %request.id,
            channel = %request.channel,
            namespace = %namespace,
            "dispatching IPC request"
        );

        match self.handlers.get(namespace) {
            Some(handler) => handler(request),
            None => {
                warn!(
                    id = %request.id,
                    channel = %request.channel,
                    "no handler registered for channel"
                );
                IpcResponse::error(
                    &request.id,
                    format!("no handler registered for channel: {}", request.channel),
                )
            }
        }
    }
}

impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::protocol::IpcRequest;

    #[test]
    fn dispatch_to_registered_handler() {
        let mut bridge = Bridge::new();
        bridge.register_handler(
            "echo",
            Box::new(|req| IpcResponse::ok(&req.id, req.payload.clone())),
        );

        let req = IpcRequest::new("echo.ping", serde_json::json!({"msg": "hello"}));
        let res = bridge.dispatch(&req);
        assert!(res.ok);
        assert_eq!(res.payload["msg"], "hello");
    }

    #[test]
    fn dispatch_unknown_channel_returns_error() {
        let bridge = Bridge::new();
        let req = IpcRequest::new("unknown.command", serde_json::json!({}));
        let res = bridge.dispatch(&req);
        assert!(!res.ok);
    }
}
