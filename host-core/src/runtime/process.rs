//! Sidecar process management.
//!
//! Handles spawning, monitoring, and terminating external processes
//! that serve as backend runtime adapters.

use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc as async_mpsc, oneshot};
use tracing::{error, info, warn};

use crate::ipc::protocol::{IpcRequest, IpcResponse};
use crate::runtime::adapter::{
    AdapterConfig, AdapterError, AdapterStatus, RuntimeAdapter, TransportKind,
};

const MAX_FRAME_BYTES: usize = 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

struct Exchange {
    message: String,
    id: String,
    response: mpsc::SyncSender<Result<String, AdapterError>>,
}

struct Session {
    requests: async_mpsc::Sender<Exchange>,
    shutdown: Option<oneshot::Sender<()>>,
    worker: JoinHandle<Result<(), AdapterError>>,
}

/// A stdio-based sidecar process adapter.
///
/// Launches an external process and communicates over stdin/stdout
/// using newline-delimited JSON messages.
pub struct StdioProcess {
    config: AdapterConfig,
    session: Option<Mutex<Session>>,
    status: Arc<Mutex<AdapterStatus>>,
    timeout: Duration,
    restart_count: u32,
    max_restarts: u32,
    last_start: Option<Instant>,
}

impl StdioProcess {
    pub fn new(config: AdapterConfig) -> Self {
        Self {
            config,
            session: None,
            status: Arc::new(Mutex::new(AdapterStatus::Stopped)),
            timeout: REQUEST_TIMEOUT,
            restart_count: 0,
            max_restarts: 3,
            last_start: None,
        }
    }

    /// Check if the sidecar process is still alive.
    /// Returns true if healthy, false if crashed or exited.
    pub fn health_check(&mut self) -> bool {
        self.status() == AdapterStatus::Running
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
        self.stop()?;
        self.start()
    }
}

impl RuntimeAdapter for StdioProcess {
    fn start(&mut self) -> Result<(), AdapterError> {
        if self.health_check() {
            return Ok(());
        }

        self.stop()?;
        if self.config.transport != TransportKind::Stdio {
            return Err(AdapterError::StartFailed(
                "StdioProcess requires stdio transport".into(),
            ));
        }
        set_status(&self.status, AdapterStatus::Starting);
        info!(
            runtime = %self.config.runtime_type,
            entry = %self.config.entry,
            "starting sidecar process"
        );

        let mut cmd = if self.config.runtime_type == "rust" {
            let mut command = Command::new("cargo");
            command.args(["run", "--quiet", "--manifest-path", &self.config.entry]);
            command
        } else {
            let mut command = Command::new(resolve_program(&self.config.runtime_type));
            command.arg(&self.config.entry);
            command
        };
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        let (requests, receiver) = async_mpsc::channel(1);
        let (shutdown, stopped) = oneshot::channel();
        let (ready, started) = mpsc::sync_channel(1);
        let status = Arc::clone(&self.status);
        let timeout = self.timeout;
        let worker = std::thread::Builder::new()
            .name("gateorix-sidecar".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let _ = ready.send(Err(error.to_string()));
                        return Ok(());
                    }
                };
                runtime.block_on(async move {
                    let mut command = tokio::process::Command::from(cmd);
                    command.kill_on_drop(true);
                    match command.spawn() {
                        Ok(child) => {
                            set_status(&status, AdapterStatus::Running);
                            let _ = ready.send(Ok(()));
                            run_session(child, receiver, stopped, status, timeout).await
                        }
                        Err(error) => {
                            let _ = ready.send(Err(error.to_string()));
                            Ok(())
                        }
                    }
                })
            })
            .map_err(|error| {
                set_status(&self.status, AdapterStatus::Crashed(error.to_string()));
                AdapterError::StartFailed(error.to_string())
            })?;
        match started.recv() {
            Ok(Ok(())) => {
                self.session = Some(Mutex::new(Session {
                    requests,
                    shutdown: Some(shutdown),
                    worker,
                }));
                self.last_start = Some(Instant::now());
                Ok(())
            }
            result => {
                let _ = worker.join();
                let message = match result {
                    Ok(Err(message)) => message,
                    _ => "sidecar worker exited during startup".into(),
                };
                set_status(&self.status, AdapterStatus::Crashed(message.clone()));
                Err(AdapterError::StartFailed(message))
            }
        }
    }

    fn stop(&mut self) -> Result<(), AdapterError> {
        if let Some(session) = self.session.take() {
            info!("stopping sidecar process");
            let mut session = session
                .into_inner()
                .unwrap_or_else(|error| error.into_inner());
            if let Some(shutdown) = session.shutdown.take() {
                let _ = shutdown.send(());
            }
            let result = session.worker.join().unwrap_or_else(|_| {
                Err(AdapterError::Communication(
                    "sidecar worker panicked".into(),
                ))
            });
            if let Err(error) = result {
                set_status(&self.status, AdapterStatus::Crashed(error.to_string()));
                return Err(error);
            }
        }
        set_status(&self.status, AdapterStatus::Stopped);
        Ok(())
    }

    fn send(&self, message: &str) -> Result<String, AdapterError> {
        let session = self.session.as_ref().ok_or(AdapterError::NotRunning)?;
        if self.status() != AdapterStatus::Running {
            return Err(AdapterError::NotRunning);
        }
        if message.len() > MAX_FRAME_BYTES {
            return Err(AdapterError::Communication("request exceeds 1 MiB".into()));
        }
        let request: IpcRequest = serde_json::from_str(message)
            .map_err(|error| AdapterError::Communication(format!("invalid request: {error}")))?;
        if request.id.is_empty() || request.channel.is_empty() || !request.payload.is_object() {
            return Err(AdapterError::Communication(
                "request requires nonempty id/channel and object payload".into(),
            ));
        }
        let message = serde_json::to_string(&request)
            .map_err(|error| AdapterError::Communication(error.to_string()))?;
        if message.len() > MAX_FRAME_BYTES {
            return Err(AdapterError::Communication(
                "encoded request exceeds 1 MiB".into(),
            ));
        }
        let mut session = session
            .try_lock()
            .map_err(|_| AdapterError::Communication("another request is in flight".into()))?;
        let (response, received) = mpsc::sync_channel(1);
        session
            .requests
            .try_send(Exchange {
                message,
                id: request.id,
                response,
            })
            .map_err(|_| AdapterError::Communication("sidecar worker unavailable".into()))?;
        match received.recv_timeout(self.timeout + Duration::from_secs(1)) {
            Ok(result) => result,
            Err(error) => {
                if let Some(shutdown) = session.shutdown.take() {
                    let _ = shutdown.send(());
                }
                let message = format!("sidecar worker: {error}");
                set_status(&self.status, AdapterStatus::Crashed(message.clone()));
                Err(AdapterError::Communication(message))
            }
        }
    }

    fn status(&self) -> AdapterStatus {
        self.status
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    fn runtime_type(&self) -> &str {
        &self.config.runtime_type
    }
}

fn set_status(status: &Mutex<AdapterStatus>, value: AdapterStatus) {
    *status.lock().unwrap_or_else(|error| error.into_inner()) = value;
}

async fn run_session(
    mut child: tokio::process::Child,
    mut requests: async_mpsc::Receiver<Exchange>,
    mut shutdown: oneshot::Receiver<()>,
    status: Arc<Mutex<AdapterStatus>>,
    timeout: Duration,
) -> Result<(), AdapterError> {
    let mut stdin = child.stdin.take().expect("piped stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
    let mut stderr = child.stderr.take().expect("piped stderr");
    let drain = tokio::spawn(async move {
        let _ = tokio::io::copy(&mut stderr, &mut tokio::io::sink()).await;
    });
    let failure = loop {
        tokio::select! {
            biased;
            _ = &mut shutdown => break None,
            exit = child.wait() => break Some(format!("sidecar exited: {exit:?}")),
            request = requests.recv() => {
                let Some(request) = request else { break None };
                let result = tokio::select! {
                    biased;
                    _ = &mut shutdown => Err(AdapterError::Communication("sidecar stopped".into())),
                    result = tokio::time::timeout(timeout, exchange(&mut stdin, &mut stdout, &request)) => {
                        result.unwrap_or_else(|_| Err(AdapterError::Communication("sidecar request timed out".into())))
                    }
                };
                if let Err(error) = &result {
                    let message = error.to_string();
                    set_status(&status, AdapterStatus::Crashed(message.clone()));
                    let cleanup = child.kill().await;
                    drain.abort();
                    let _ = drain.await;
                    let _ = request.response.send(result);
                    return cleanup.map_err(|error| AdapterError::Communication(format!("sidecar cleanup failed: {error}")));
                }
                let _ = request.response.send(result);
            }
        }
    };
    drop(stdin);
    let cleanup = child.kill().await;
    drain.abort();
    let _ = drain.await;
    if let Some(message) = failure {
        set_status(&status, AdapterStatus::Crashed(message));
    } else if let Err(error) = &cleanup {
        set_status(
            &status,
            AdapterStatus::Crashed(format!("sidecar cleanup failed: {error}")),
        );
    } else {
        set_status(&status, AdapterStatus::Stopped);
    }
    cleanup.map_err(|error| AdapterError::Communication(format!("sidecar cleanup failed: {error}")))
}

async fn exchange(
    stdin: &mut tokio::process::ChildStdin,
    stdout: &mut BufReader<tokio::process::ChildStdout>,
    request: &Exchange,
) -> Result<String, AdapterError> {
    let io_error = |error: std::io::Error| AdapterError::Communication(error.to_string());
    stdin
        .write_all(request.message.as_bytes())
        .await
        .map_err(io_error)?;
    stdin.write_all(b"\n").await.map_err(io_error)?;
    stdin.flush().await.map_err(io_error)?;
    let mut frame = Vec::new();
    loop {
        let buffer = stdout.fill_buf().await.map_err(io_error)?;
        if buffer.is_empty() {
            return Err(AdapterError::ProcessExited(
                "stdout closed before a complete response".into(),
            ));
        }
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let length = newline.unwrap_or(buffer.len());
        if frame.len() + length > MAX_FRAME_BYTES {
            return Err(AdapterError::Communication("response exceeds 1 MiB".into()));
        }
        frame.extend_from_slice(&buffer[..length]);
        stdout.consume(length + usize::from(newline.is_some()));
        if newline.is_some() {
            break;
        }
    }
    let envelope: serde_json::Value = serde_json::from_slice(&frame)
        .map_err(|error| AdapterError::Communication(format!("invalid response: {error}")))?;
    if !envelope
        .as_object()
        .is_some_and(|object| object.contains_key("payload"))
    {
        return Err(AdapterError::Communication(
            "response requires a payload field".into(),
        ));
    }
    let response: IpcResponse = serde_json::from_value(envelope)
        .map_err(|error| AdapterError::Communication(format!("invalid response: {error}")))?;
    if response.id != request.id {
        return Err(AdapterError::Communication(
            "response id does not match request".into(),
        ));
    }
    String::from_utf8(frame).map_err(|error| AdapterError::Communication(error.to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::adapter::TransportKind;
    use std::sync::OnceLock;

    const REQUEST: &str = r#"{"id":"test","channel":"runtime.greet","payload":{}}"#;

    fn fixture(mode: &str) -> StdioProcess {
        static FIXTURE: OnceLock<tempfile::TempDir> = OnceLock::new();
        let directory = FIXTURE.get_or_init(|| {
            let directory = tempfile::Builder::new()
                .prefix("gateorix sidecar ; ")
                .tempdir()
                .unwrap();
            let output = Command::new("rustc")
                .arg(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/stdio-sidecar.rs"
                ))
                .arg("-o")
                .arg(directory.path().join("sidecar.exe"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            directory
        });
        StdioProcess::new(AdapterConfig {
            runtime_type: directory
                .path()
                .join("sidecar.exe")
                .to_string_lossy()
                .into_owned(),
            entry: mode.into(),
            transport: TransportKind::Stdio,
            port: None,
            env: Default::default(),
        })
    }

    #[test]
    fn sends_requests_and_reads_correlated_responses() {
        let mut adapter = fixture("echo");
        adapter.start().unwrap();
        for _ in 0..2 {
            let response = adapter.send(REQUEST).unwrap();
            let response: serde_json::Value = serde_json::from_str(&response).unwrap();
            assert_eq!(response["id"], "test");
            assert_eq!(response["payload"]["message"], "hello");
        }
        adapter.stop().unwrap();
        assert_eq!(adapter.status(), AdapterStatus::Stopped);
    }

    #[test]
    fn drains_stderr_without_blocking_responses() {
        let mut adapter = fixture("stderr");
        adapter.start().unwrap();
        assert!(adapter.send(REQUEST).is_ok());
        adapter.stop().unwrap();
    }

    #[test]
    fn invalid_responses_and_crashes_invalidate_the_session() {
        for mode in [
            "wrong-id",
            "malformed",
            "bad-shape",
            "missing-payload",
            "oversized",
            "truncated",
            "crash",
        ] {
            let mut adapter = fixture(mode);
            adapter.start().unwrap();
            assert!(adapter.send(REQUEST).is_err(), "{mode}");
            assert!(
                matches!(adapter.status(), AdapterStatus::Crashed(_)),
                "{mode}"
            );
            assert!(matches!(
                adapter.send(REQUEST),
                Err(AdapterError::NotRunning)
            ));
            adapter.stop().unwrap();
        }
    }

    #[test]
    fn deadlines_cover_reading_and_writing() {
        for mode in ["silent", "no-read"] {
            let mut adapter = fixture(mode);
            adapter.timeout = Duration::from_millis(200);
            adapter.start().unwrap();
            let request = serde_json::json!({
                "id": "test", "channel": "runtime.greet",
                "payload": { "text": "x".repeat(512 * 1024) }
            })
            .to_string();
            let started = Instant::now();
            let error = adapter.send(&request).unwrap_err();
            assert!(error.to_string().contains("timed out"), "{mode}: {error}");
            assert!(started.elapsed() < Duration::from_secs(3));
            assert!(matches!(adapter.status(), AdapterStatus::Crashed(_)));
            adapter.stop().unwrap();
        }
    }

    #[test]
    fn rejects_bad_requests_without_poisoning_session() {
        let mut adapter = fixture("echo");
        adapter.start().unwrap();
        for request in [
            "null",
            "[]",
            "{}",
            r#"{"id":"","channel":"test","payload":{}}"#,
            r#"{"id":"test","channel":"test","payload":[]}"#,
        ] {
            assert!(adapter.send(request).is_err());
        }
        assert!(adapter.send(&"x".repeat(MAX_FRAME_BYTES + 1)).is_err());
        assert!(adapter.send(REQUEST).is_ok());
        adapter.stop().unwrap();
    }

    #[test]
    fn repeated_start_stop_and_restart_clean_up_workers() {
        let mut adapter = fixture("echo");
        assert!(matches!(
            adapter.send(REQUEST),
            Err(AdapterError::NotRunning)
        ));
        for _ in 0..3 {
            adapter.start().unwrap();
            adapter.start().unwrap();
            assert!(adapter.send(REQUEST).is_ok());
            adapter.stop().unwrap();
            adapter.stop().unwrap();
        }
        adapter.start().unwrap();
        adapter.try_restart().unwrap();
        assert!(adapter.send(REQUEST).is_ok());
    }

    #[test]
    fn missing_executable_and_http_transport_fail_startup() {
        let mut adapter = fixture("echo");
        adapter.config.runtime_type.push_str("-missing");
        assert!(matches!(adapter.start(), Err(AdapterError::StartFailed(_))));
        assert!(matches!(adapter.status(), AdapterStatus::Crashed(_)));
        adapter.config.transport = TransportKind::Http;
        assert!(adapter.start().is_err());
    }

    #[test]
    fn preserves_buffered_frames_between_requests() {
        let mut adapter = fixture("buffered");
        adapter.start().unwrap();
        assert!(adapter.send(REQUEST).unwrap().contains("test"));
        let request = REQUEST.replace("test", "next");
        assert!(adapter.send(&request).unwrap().contains("next"));
        adapter.stop().unwrap();
    }

    #[test]
    fn concurrent_requests_fail_without_waiting_for_the_first() {
        let mut adapter = fixture("silent");
        adapter.timeout = Duration::from_millis(500);
        adapter.start().unwrap();
        std::thread::scope(|scope| {
            let pending = scope.spawn(|| adapter.send(REQUEST));
            let started = Instant::now();
            while adapter.session.as_ref().unwrap().try_lock().is_ok() {
                assert!(started.elapsed() < Duration::from_secs(3));
                std::thread::yield_now();
            }
            let error = adapter.send(REQUEST).unwrap_err();
            assert!(error.to_string().contains("in flight"));
            assert!(pending.join().unwrap().is_err());
        });
        adapter.stop().unwrap();
    }

    #[test]
    fn observes_exit_without_a_request() {
        let mut adapter = fixture("idle-exit");
        adapter.start().unwrap();
        let started = Instant::now();
        while adapter.health_check() {
            assert!(started.elapsed() < Duration::from_secs(3));
            std::thread::yield_now();
        }
        assert!(matches!(adapter.status(), AdapterStatus::Crashed(_)));
        adapter.stop().unwrap();
    }

    #[test]
    fn stop_and_drop_release_child_resources() {
        for explicit_stop in [true, false] {
            let mut adapter = fixture("lifecycle");
            adapter.start().unwrap();
            let response: serde_json::Value =
                serde_json::from_str(&adapter.send(REQUEST).unwrap()).unwrap();
            let port = response["payload"]["port"].as_u64().unwrap() as u16;
            let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
            let connection =
                std::net::TcpStream::connect_timeout(&address, Duration::from_secs(1)).unwrap();
            drop(connection);
            if explicit_stop {
                adapter.stop().unwrap();
            }
            drop(adapter);
            assert!(
                std::net::TcpStream::connect_timeout(&address, Duration::from_millis(200)).is_err()
            );
        }
    }

    #[test]
    fn application_errors_keep_the_transport_usable() {
        let mut adapter = fixture("application-error");
        adapter.start().unwrap();
        for _ in 0..2 {
            let response: IpcResponse =
                serde_json::from_str(&adapter.send(REQUEST).unwrap()).unwrap();
            assert!(!response.ok);
            assert_eq!(adapter.status(), AdapterStatus::Running);
        }
        adapter.stop().unwrap();
    }
}
