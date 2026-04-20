//! Sidecar process management.
//!
//! Handles spawning, monitoring, and terminating external processes
//! that serve as backend runtime adapters.

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use crate::runtime::adapter::{AdapterConfig, AdapterError, AdapterStatus, RuntimeAdapter};

/// A stdio-based sidecar process adapter.
///
/// Launches an external process and communicates over stdin/stdout
/// using newline-delimited JSON messages.
pub struct StdioProcess {
    config: AdapterConfig,
    child: Option<Child>,
    status: AdapterStatus,
    restart_count: u32,
    max_restarts: u32,
    last_start: Option<Instant>,
}

impl StdioProcess {
    pub fn new(config: AdapterConfig) -> Self {
        Self {
            config,
            child: None,
            status: AdapterStatus::Stopped,
            restart_count: 0,
            max_restarts: 3,
            last_start: None,
        }
    }

    /// Check if the sidecar process is still alive.
    /// Returns true if healthy, false if crashed or exited.
    pub fn health_check(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(exit)) => {
                    let msg = format!("sidecar exited with status: {exit}");
                    warn!("{}", msg);
                    self.status = AdapterStatus::Crashed(msg);
                    self.child = None;
                    false
                }
                Ok(None) => true, // still running
                Err(e) => {
                    let msg = format!("health check failed: {e}");
                    warn!("{}", msg);
                    self.status = AdapterStatus::Crashed(msg);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Attempt to restart the sidecar if it has crashed.
    /// Respects max_restarts limit and enforces a minimum uptime
    /// before counting a restart (to avoid rapid restart loops).
    pub fn try_restart(&mut self) -> Result<(), AdapterError> {
        if self.restart_count >= self.max_restarts {
            error!(
                "sidecar exceeded max restarts ({}), not restarting",
                self.max_restarts
            );
            return Err(AdapterError::ProcessExited(format!(
                "exceeded max restarts ({})",
                self.max_restarts
            )));
        }

        // If the process ran for less than 5 seconds, count it as a fast crash
        if let Some(last) = self.last_start {
            if last.elapsed() < Duration::from_secs(5) {
                self.restart_count += 1;
                warn!(
                    "sidecar crashed quickly (restart {}/{})",
                    self.restart_count, self.max_restarts
                );
            } else {
                // Reset counter if it ran long enough
                self.restart_count = 0;
            }
        }

        info!("restarting sidecar process...");
        self.child = None;
        self.start()
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
                self.last_start = Some(Instant::now());
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
        let child = self.child.as_ref().ok_or(AdapterError::NotRunning)?;

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
