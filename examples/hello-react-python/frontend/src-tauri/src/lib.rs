use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{Manager, State};

/// Holds the Python sidecar process so we can send/receive messages.
struct Sidecar(Mutex<Option<Child>>);

/// Tauri command: invoke the Python backend via stdio IPC.
/// The frontend calls this instead of the HTTP bridge.
#[tauri::command]
fn invoke_backend(
    channel: String,
    payload: serde_json::Value,
    sidecar: State<'_, Sidecar>,
) -> Result<serde_json::Value, String> {
    let mut guard = sidecar.0.lock().map_err(|e| e.to_string())?;
    let child = guard.as_mut().ok_or("sidecar not running")?;

    // Build the IPC request
    let request = serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "channel": channel,
        "payload": payload,
    });

    // Write request to sidecar's stdin
    let stdin = child.stdin.as_mut().ok_or("no stdin")?;
    let msg = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    writeln!(stdin, "{}", msg).map_err(|e| e.to_string())?;
    stdin.flush().map_err(|e| e.to_string())?;

    // Read one line of response from sidecar's stdout
    let stdout = child.stdout.as_mut().ok_or("no stdout")?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("read error: {}", e))?;

    if line.is_empty() {
        return Err("sidecar returned empty response (process may have exited)".into());
    }

    let response: serde_json::Value = serde_json::from_str(line.trim())
        .map_err(|e| format!("JSON parse error: {} (raw: {:?})", e, line))?;
    Ok(response)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Sidecar(Mutex::new(None)))
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![invoke_backend])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Resolve backend path relative to the Cargo manifest dir (src-tauri/)
            // so it works regardless of the exe's working directory.
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let backend_script = manifest_dir
                .join("..")
                .join("..")
                .join("backend")
                .join("main.py");
            let backend_script = std::fs::canonicalize(&backend_script).map_err(|e| {
                format!(
                    "Cannot find backend/main.py: {} (looked at {:?})",
                    e, backend_script
                )
            })?;

            log::info!("Starting Python sidecar: {:?}", backend_script);

            let python_child = Command::new("python")
                .arg(&backend_script)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to start Python sidecar: {}", e))?;

            // Store the child in managed state
            let sidecar: State<'_, Sidecar> = app.state();
            *sidecar.0.lock().unwrap() = Some(python_child);

            log::info!("Python sidecar started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
