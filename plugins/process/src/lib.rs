//! Process plugin for Gateorix.
//!
//! Provides permission-gated command execution. Only allowed when the
//! manifest explicitly grants "process" permission.

use gateorix_host_core::ipc::protocol::{IpcRequest, IpcResponse};
use gateorix_host_core::plugins::Plugin;
use std::process::Command;
use tracing::info;

pub struct ProcessPlugin;

impl ProcessPlugin {
    pub fn new() -> Self {
        Self
    }

    fn execute(&self, request: &IpcRequest) -> IpcResponse {
        let program = match request.payload.get("program").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return IpcResponse::error(&request.id, "missing 'program' in payload"),
        };

        let args: Vec<&str> = request
            .payload
            .get("args")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect()
            })
            .unwrap_or_default();

        match Command::new(program).args(&args).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                IpcResponse::ok(
                    &request.id,
                    serde_json::json!({
                        "exitCode": output.status.code(),
                        "stdout": stdout,
                        "stderr": stderr,
                    }),
                )
            }
            Err(e) => IpcResponse::error(&request.id, format!("execution failed: {}", e)),
        }
    }
}

impl Default for ProcessPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ProcessPlugin {
    fn namespace(&self) -> &str {
        "process"
    }

    fn handle(&self, request: &IpcRequest) -> IpcResponse {
        let action = request.channel.strip_prefix("process.").unwrap_or("");
        match action {
            "execute" => self.execute(request),
            _ => IpcResponse::error(
                &request.id,
                format!("unknown process action: {}", action),
            ),
        }
    }

    fn on_init(&self) {
        info!("process plugin initialized");
    }
}
