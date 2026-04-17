//! Filesystem plugin for Gateorix.
//!
//! Provides scoped file operations (read, write, list, exists) gated
//! by the application's permission manifest.

use gateorix_host_core::ipc::protocol::{IpcRequest, IpcResponse};
use gateorix_host_core::plugins::Plugin;
use std::path::PathBuf;
use tracing::info;

pub struct FilesystemPlugin {
    /// Base directory for resolving relative paths.
    base_dir: PathBuf,
}

impl FilesystemPlugin {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn read_text(&self, request: &IpcRequest) -> IpcResponse {
        let path = match request.payload.get("path").and_then(|p| p.as_str()) {
            Some(p) => self.base_dir.join(p),
            None => return IpcResponse::error(&request.id, "missing 'path' in payload"),
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => IpcResponse::ok(&request.id, serde_json::json!({ "content": content })),
            Err(e) => IpcResponse::error(&request.id, format!("read failed: {}", e)),
        }
    }

    fn write_text(&self, request: &IpcRequest) -> IpcResponse {
        let path = match request.payload.get("path").and_then(|p| p.as_str()) {
            Some(p) => self.base_dir.join(p),
            None => return IpcResponse::error(&request.id, "missing 'path' in payload"),
        };

        let content = match request.payload.get("content").and_then(|c| c.as_str()) {
            Some(c) => c,
            None => return IpcResponse::error(&request.id, "missing 'content' in payload"),
        };

        match std::fs::write(&path, content) {
            Ok(_) => IpcResponse::ok(&request.id, serde_json::json!({ "written": true })),
            Err(e) => IpcResponse::error(&request.id, format!("write failed: {}", e)),
        }
    }

    fn exists(&self, request: &IpcRequest) -> IpcResponse {
        let path = match request.payload.get("path").and_then(|p| p.as_str()) {
            Some(p) => self.base_dir.join(p),
            None => return IpcResponse::error(&request.id, "missing 'path' in payload"),
        };

        IpcResponse::ok(&request.id, serde_json::json!({ "exists": path.exists() }))
    }

    fn list_dir(&self, request: &IpcRequest) -> IpcResponse {
        let path = match request.payload.get("path").and_then(|p| p.as_str()) {
            Some(p) => self.base_dir.join(p),
            None => return IpcResponse::error(&request.id, "missing 'path' in payload"),
        };

        match std::fs::read_dir(&path) {
            Ok(entries) => {
                let names: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();
                IpcResponse::ok(&request.id, serde_json::json!({ "entries": names }))
            }
            Err(e) => IpcResponse::error(&request.id, format!("list failed: {}", e)),
        }
    }
}

impl Plugin for FilesystemPlugin {
    fn namespace(&self) -> &str {
        "filesystem"
    }

    fn handle(&self, request: &IpcRequest) -> IpcResponse {
        let action = request.channel.strip_prefix("filesystem.").unwrap_or("");
        match action {
            "readText" => self.read_text(request),
            "writeText" => self.write_text(request),
            "exists" => self.exists(request),
            "listDir" => self.list_dir(request),
            _ => IpcResponse::error(
                &request.id,
                format!("unknown filesystem action: {}", action),
            ),
        }
    }

    fn on_init(&self) {
        info!(base_dir = %self.base_dir.display(), "filesystem plugin initialized");
    }
}
