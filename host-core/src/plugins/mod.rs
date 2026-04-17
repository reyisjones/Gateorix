//! Plugin system.
//!
//! Provides the trait and registry for host-side plugins. Plugins expose
//! OS capabilities (filesystem, clipboard, notifications, etc.) to the
//! IPC bridge as named channels.

use crate::ipc::protocol::{IpcRequest, IpcResponse};

/// Trait that all host plugins must implement.
pub trait Plugin: Send + Sync {
    /// The channel namespace this plugin handles (e.g. "filesystem").
    fn namespace(&self) -> &str;

    /// Handle an incoming IPC request on this plugin's channel.
    fn handle(&self, request: &IpcRequest) -> IpcResponse;

    /// Called once when the plugin is registered with the host.
    fn on_init(&self) {}

    /// Called when the host is shutting down.
    fn on_shutdown(&self) {}
}

/// Registry that holds all active plugins.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin. Panics if a plugin with the same namespace
    /// is already registered.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let ns = plugin.namespace().to_string();
        if self.plugins.iter().any(|p| p.namespace() == ns) {
            panic!("duplicate plugin namespace: {}", ns);
        }
        plugin.on_init();
        self.plugins.push(plugin);
    }

    /// Find a plugin by namespace.
    pub fn get(&self, namespace: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| p.namespace() == namespace)
            .map(|p| p.as_ref())
    }

    /// Shut down all plugins.
    pub fn shutdown_all(&self) {
        for plugin in &self.plugins {
            plugin.on_shutdown();
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
