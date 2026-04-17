//! Sidecar process management.
//!
//! Handles spawning, monitoring, and terminating external processes
//! that serve as backend runtime adapters.

use std::process::{Child, Command, Stdio};
use tracing::{error, info};

use crate::runtime::adapter::{AdapterConfig, AdapterError, AdapterStatus, RuntimeAdapter};

/// A stdio-based sidecar process adapter.
///
/// Launches an external process and communicates over stdin/stdout
/// using newline-delimited JSON messages.
pub struct StdioProcess {
    config: AdapterConfig,
    child: Option<Child>,
    status: AdapterStatus,
}

impl StdioProcess {
    pub fn new(config: AdapterConfig) -> Self {
        Self {
            config,
            child: None,
            status: AdapterStatus::Stopped,
        }
    }
}

impl RuntimeAdapter for StdioProcess {
    fn start(&mut self) -> Result<(), AdapterError> {
        if self.child.is_some() {
            return Ok(());
        }

        self.status = AdapterStatus::Starting;
        info!(
            runtime = %self.config.runtime_type,
            entry = %self.config.entry,
            "starting sidecar process"
        );

        let program = resolve_program(&self.config.runtime_type);
        let mut cmd = Command::new(&program);
        cmd.arg(&self.config.entry)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        match cmd.spawn() {
            Ok(child) => {
                self.child = Some(child);
                self.status = AdapterStatus::Running;
                info!("sidecar process started");
                Ok(())
            }
            Err(e) => {
                self.status = AdapterStatus::Crashed(e.to_string());
                error!(error = %e, "failed to start sidecar");
                Err(AdapterError::StartFailed(e.to_string()))
            }
        }
    }

    fn stop(&mut self) -> Result<(), AdapterError> {
        if let Some(mut child) = self.child.take() {
            info!("stopping sidecar process");
            let _ = child.kill();
            let _ = child.wait();
            self.status = AdapterStatus::Stopped;
        }
        Ok(())
    }

    fn send(&self, message: &str) -> Result<String, AdapterError> {
        let child = self
            .child
            .as_ref()
            .ok_or(AdapterError::NotRunning)?;

        // Placeholder: the production implementation will use async I/O
        // with tokio channels for concurrent read/write on the sidecar's
        // stdin/stdout. For now, verify the process is alive.
        let _ = message;

        if child.stdin.is_none() {
            return Err(AdapterError::Communication("stdin not available".into()));
        }

        Ok(String::new())
    }

    fn status(&self) -> AdapterStatus {
        self.status.clone()
    }

    fn runtime_type(&self) -> &str {
        &self.config.runtime_type
    }
}

impl Drop for StdioProcess {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Resolve the system command for a given runtime type.
fn resolve_program(runtime_type: &str) -> String {
    match runtime_type {
        "python" => "python3".into(),
        "go" => "go".into(),
        "dotnet" => "dotnet".into(),
        "swift" => "swift".into(),
        other => other.into(),
    }
}
