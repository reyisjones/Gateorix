//! IPC bridge dispatcher.
//!
//! Routes incoming IPC requests to the appropriate handler —
//! either a host-core plugin or a runtime adapter.

use crate::ipc::protocol::{IpcRequest, IpcResponse};
use crate::permissions::{Capability, PermissionGuard, PermissionResult};
use crate::plugins::Plugin;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

/// Handler function type for IPC channels.
pub type ChannelHandler = Box<dyn Fn(&IpcRequest) -> IpcResponse + Send + Sync>;

/// The IPC bridge routes frontend requests to registered channel handlers.
pub struct Bridge {
    handlers: HashMap<String, ChannelHandler>,
    permissions: Option<Arc<PermissionGuard>>,
}

impl Bridge {
    /// Create a new empty bridge.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            permissions: None,
        }
    }

    pub fn with_permissions(permissions: PermissionGuard) -> Self {
        Self {
            handlers: HashMap::new(),
            permissions: Some(Arc::new(permissions)),
        }
    }

    /// Register a handler for the given channel prefix.
    ///
    /// For example, registering "filesystem" will handle channels like
    /// "filesystem.readText", "filesystem.writeText", etc.
    pub fn register_handler(&mut self, channel: impl Into<String>, handler: ChannelHandler) {
        self.handlers.insert(channel.into(), handler);
    }

    pub fn register_plugin(&mut self, plugin: impl Plugin + 'static) {
        let namespace = plugin.namespace().to_owned();
        let permissions = self.permissions.clone();
        plugin.on_init();
        self.register_handler(
            namespace,
            Box::new(move |request| match &permissions {
                Some(permissions) => plugin.handle(request, permissions),
                None => IpcResponse::error(&request.id, "no permissions configured"),
            }),
        );
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

        let capability = match namespace {
            "filesystem" => match request.payload.get("path").and_then(|value| value.as_str()) {
                Some(path) => Some(Capability::Filesystem(path.into())),
                None => return IpcResponse::error(&request.id, "missing filesystem path"),
            },
            "process" => Some(Capability::Process),
            "notifications" => Some(Capability::Notifications),
            "clipboard" => Some(Capability::Clipboard),
            _ => None,
        };
        if let Some(capability) = capability {
            match &self.permissions {
                Some(permissions) => {
                    if let PermissionResult::Denied(reason) = permissions.check(&capability) {
                        return IpcResponse::error(&request.id, reason);
                    }
                }
                None => return IpcResponse::error(&request.id, "no permissions configured"),
            }
        }

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

    #[test]
    fn privileged_handlers_are_not_called_without_permissions() {
        for namespace in ["filesystem", "process", "notifications", "clipboard"] {
            let mut bridge = Bridge::new();
            bridge.register_handler(namespace, Box::new(|_| panic!("denied handler executed")));
            let request = IpcRequest::new(
                format!("{namespace}.execute"),
                serde_json::json!({"path": "data/test.txt"}),
            );
            assert!(!bridge.dispatch(&request).ok);
        }
    }

    #[test]
    fn manifest_denial_prevents_process_execution() {
        let config = crate::app::PermissionConfig {
            filesystem: vec![],
            process: false,
            notifications: false,
            clipboard: false,
        };
        let mut bridge = Bridge::with_permissions(PermissionGuard::new(config, ".".into()));
        bridge.register_handler("process", Box::new(|_| panic!("denied process executed")));
        assert!(
            !bridge
                .dispatch(&IpcRequest::new("process.execute", serde_json::json!({})))
                .ok
        );
    }
}
