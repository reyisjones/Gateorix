//! Filesystem plugin for Gateorix.
//!
//! Provides scoped file operations (read, write, list, exists) gated
//! by the application's permission manifest.

use gateorix_host_core::ipc::protocol::{IpcRequest, IpcResponse};
use gateorix_host_core::permissions::{PermissionGuard, ScopedFilesystem};
use gateorix_host_core::plugins::Plugin;
use std::path::Path;

#[derive(Default)]
pub struct FilesystemPlugin;

impl FilesystemPlugin {
    pub fn new() -> Self {
        Self
    }

    fn read_text(&self, request: &IpcRequest, scope: &ScopedFilesystem<'_>) -> IpcResponse {
        match scope.read_text() {
            Ok(content) => IpcResponse::ok(&request.id, serde_json::json!({ "content": content })),
            Err(e) => IpcResponse::error(&request.id, format!("read failed: {}", e)),
        }
    }

    fn write_text(&self, request: &IpcRequest, scope: &ScopedFilesystem<'_>) -> IpcResponse {
        let content = match request.payload.get("content").and_then(|c| c.as_str()) {
            Some(c) => c,
            None => return IpcResponse::error(&request.id, "missing 'content' in payload"),
        };

        match scope.write_text(content) {
            Ok(_) => IpcResponse::ok(&request.id, serde_json::json!({ "written": true })),
            Err(e) => IpcResponse::error(&request.id, format!("write failed: {}", e)),
        }
    }

    fn exists(&self, request: &IpcRequest, scope: &ScopedFilesystem<'_>) -> IpcResponse {
        match scope.exists() {
            Ok(exists) => IpcResponse::ok(&request.id, serde_json::json!({ "exists": exists })),
            Err(error) => IpcResponse::error(&request.id, error.to_string()),
        }
    }

    fn list_dir(&self, request: &IpcRequest, scope: &ScopedFilesystem<'_>) -> IpcResponse {
        match scope.list_dir() {
            Ok(entries) => IpcResponse::ok(&request.id, serde_json::json!({ "entries": entries })),
            Err(e) => IpcResponse::error(&request.id, format!("list failed: {}", e)),
        }
    }
}

impl Plugin for FilesystemPlugin {
    fn namespace(&self) -> &str {
        "filesystem"
    }

    fn handle(&self, request: &IpcRequest, permissions: &PermissionGuard) -> IpcResponse {
        let path = match request.payload.get("path").and_then(|value| value.as_str()) {
            Some(path) => path,
            None => return IpcResponse::error(&request.id, "missing 'path' in payload"),
        };
        let scope = match permissions.filesystem(Path::new(path)) {
            Ok(scope) => scope,
            Err(error) => return IpcResponse::error(&request.id, error.to_string()),
        };
        let action = request.channel.strip_prefix("filesystem.").unwrap_or("");
        match action {
            "readText" => self.read_text(request, &scope),
            "writeText" => self.write_text(request, &scope),
            "exists" => self.exists(request, &scope),
            "listDir" => self.list_dir(request, &scope),
            _ => IpcResponse::error(
                &request.id,
                format!("unknown filesystem action: {}", action),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gateorix_host_core::{app::PermissionConfig, ipc::bridge::Bridge};

    #[test]
    fn dispatcher_and_direct_plugin_calls_enforce_scopes() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("data")).unwrap();
        let config = PermissionConfig {
            filesystem: vec!["data".into()],
            process: false,
            clipboard: false,
            notifications: false,
        };
        let mut bridge = Bridge::with_permissions(PermissionGuard::new(config, root.path().into()));
        bridge.register_plugin(FilesystemPlugin::new());
        let request = |path: &str| {
            IpcRequest::new(
                "filesystem.writeText",
                serde_json::json!({"path": path, "content": "test"}),
            )
        };
        assert!(bridge.dispatch(&request("data/allowed.txt")).ok);
        assert!(!bridge.dispatch(&request("data/../outside.txt")).ok);
        assert!(!root.path().join("outside.txt").exists());
        let denied = PermissionGuard::new(
            PermissionConfig {
                filesystem: vec![],
                process: false,
                clipboard: false,
                notifications: false,
            },
            root.path().into(),
        );
        assert!(
            !FilesystemPlugin::new()
                .handle(&request("data/denied.txt"), &denied)
                .ok
        );
        assert!(!root.path().join("data/denied.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn plugin_denies_symlink_escape_even_without_dispatcher() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("data")).unwrap();
        std::fs::write(root.path().join("secret"), "original").unwrap();
        std::os::unix::fs::symlink("../secret", root.path().join("data/link")).unwrap();
        let permissions = PermissionGuard::new(
            PermissionConfig {
                filesystem: vec!["data".into()],
                process: false,
                clipboard: false,
                notifications: false,
            },
            root.path().into(),
        );
        let request = IpcRequest::new(
            "filesystem.writeText",
            serde_json::json!({"path": "data/link", "content": "changed"}),
        );
        assert!(!FilesystemPlugin::new().handle(&request, &permissions).ok);
        assert_eq!(
            std::fs::read_to_string(root.path().join("secret")).unwrap(),
            "original"
        );
    }
}
