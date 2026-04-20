use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    Manager, State, WebviewUrl, WebviewWindowBuilder,
};

struct Sidecar(Mutex<Option<Child>>);
struct SettingsPath(PathBuf);

#[tauri::command]
fn invoke_backend(channel: String, payload: serde_json::Value, sidecar: State<'_, Sidecar>) -> Result<serde_json::Value, String> {
    let mut guard = sidecar.0.lock().map_err(|e| e.to_string())?;
    let child = guard.as_mut().ok_or("sidecar not running")?;
    let request = serde_json::json!({ "id": uuid::Uuid::new_v4().to_string(), "channel": channel, "payload": payload });
    let stdin = child.stdin.as_mut().ok_or("no stdin")?;
    writeln!(stdin, "{}", serde_json::to_string(&request).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    stdin.flush().map_err(|e| e.to_string())?;
    let stdout = child.stdout.as_mut().ok_or("no stdout")?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| format!("read error: {}", e))?;
    if line.is_empty() { return Err("sidecar returned empty response".into()); }
    serde_json::from_str(line.trim()).map_err(|e| format!("JSON parse error: {}", e))
}

#[tauri::command]
fn get_settings(settings_path: State<'_, SettingsPath>) -> Result<serde_json::Value, String> {
    let path = &settings_path.0;
    if !path.exists() { return Ok(serde_json::json!({})); }
    let contents = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(data: serde_json::Value, settings_path: State<'_, SettingsPath>) -> Result<(), String> {
    let path = &settings_path.0;
    let mut settings: serde_json::Map<String, serde_json::Value> = if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(path).map_err(|e| e.to_string())?).unwrap_or_default()
    } else { serde_json::Map::new() };
    if let serde_json::Value::Object(incoming) = data { for (k, v) in incoming { settings.insert(k, v); } }
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    std::fs::write(path, serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

#[tauri::command]
fn login(username: String, password: String) -> Result<String, String> {
    match (username.as_str(), password.as_str()) {
        ("admin", "gateorix") => Ok("Admin".into()),
        ("demo", "demo") => Ok("Demo User".into()),
        _ => Err("Invalid username or password".into()),
    }
}

#[tauri::command]
fn logout() -> Result<(), String> { Ok(()) }

#[tauri::command]
fn toggle_fullscreen(window: tauri::WebviewWindow) -> Result<(), String> {
    let fs = window.is_fullscreen().map_err(|e| e.to_string())?;
    window.set_fullscreen(!fs).map_err(|e| e.to_string())
}

#[tauri::command]
fn resize_window(window: tauri::WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height })).map_err(|e| e.to_string())
}

#[tauri::command]
fn minimize_window(window: tauri::WebviewWindow) -> Result<(), String> { window.minimize().map_err(|e| e.to_string()) }

#[tauri::command]
fn maximize_window(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.is_maximized().map_err(|e| e.to_string())? { window.unmaximize().map_err(|e| e.to_string()) }
    else { window.maximize().map_err(|e| e.to_string()) }
}

#[tauri::command]
fn open_new_window(app: tauri::AppHandle, label: String, title: String, url: String, width: f64, height: f64) -> Result<(), String> {
    let u = if url.starts_with("http") { WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?) } else { WebviewUrl::App(url.into()) };
    WebviewWindowBuilder::new(&app, &label, u).title(&title).inner_size(width, height).build().map_err(|e| e.to_string())?; Ok(())
}

#[tauri::command]
fn close_window(window: tauri::WebviewWindow) -> Result<(), String> { window.close().map_err(|e| e.to_string()) }

#[tauri::command]
async fn open_file_dialog(window: tauri::WebviewWindow) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    Ok(window.dialog().file().blocking_pick_file().map(|f| f.to_string()))
}

#[tauri::command]
async fn save_file_dialog(window: tauri::WebviewWindow) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    Ok(window.dialog().file().blocking_save_file().map(|f| f.to_string()))
}

#[tauri::command]
async fn open_folder_dialog(window: tauri::WebviewWindow) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    Ok(window.dialog().file().blocking_pick_folder().map(|f| f.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Sidecar(Mutex::new(None)))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            invoke_backend, get_settings, save_settings, login, logout,
            toggle_fullscreen, resize_window, minimize_window, maximize_window,
            open_new_window, close_window, open_file_dialog, save_file_dialog, open_folder_dialog,
        ])
        .setup(|app| {
            let file_menu = Submenu::with_items(app, "File", true, &[
                &MenuItem::with_id(app, "open_file", "Open File...", true, Some("CmdOrCtrl+O"))?,
                &MenuItem::with_id(app, "save_file", "Save As...", true, Some("CmdOrCtrl+Shift+S"))?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "quit", "Quit", true, Some("CmdOrCtrl+Q"))?,
            ])?;
            let view_menu = Submenu::with_items(app, "View", true, &[
                &MenuItem::with_id(app, "fullscreen", "Toggle Fullscreen", true, Some("F11"))?,
                &MenuItem::with_id(app, "zoom_in", "Zoom In", true, Some("CmdOrCtrl+Plus"))?,
                &MenuItem::with_id(app, "zoom_out", "Zoom Out", true, Some("CmdOrCtrl+Minus"))?,
            ])?;
            let help_menu = Submenu::with_items(app, "Help", true, &[
                &MenuItem::with_id(app, "about", "About Gateorix", true, None::<&str>)?,
                &MenuItem::with_id(app, "docs", "Documentation", true, None::<&str>)?,
            ])?;
            app.set_menu(Menu::with_items(app, &[&file_menu, &view_menu, &help_menu])?)?;

            app.on_menu_event(|app, event| match event.id().as_ref() {
                "quit" => app.exit(0),
                "fullscreen" => { if let Some(w) = app.get_webview_window("main") { let f = w.is_fullscreen().unwrap_or(false); let _ = w.set_fullscreen(!f); } }
                "about" => { if let Some(w) = app.get_webview_window("main") { let _ = w.eval("alert('Gateorix v0.1.0 — Web UI, Native Power')"); } }
                _ => {}
            });

            let tray_menu = Menu::with_items(app, &[
                &MenuItem::with_id(app, "tray_show", "Show Window", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "tray_quit", "Quit", true, None::<&str>)?,
            ])?;
            TrayIconBuilder::new().tooltip("Gateorix").menu(&tray_menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "tray_show" => { if let Some(w) = app.get_webview_window("main") { let _ = w.show(); let _ = w.set_focus(); } }
                    "tray_quit" => app.exit(0),
                    _ => {}
                }).build(app)?;

            if cfg!(debug_assertions) {
                app.handle().plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())?;
            }

            let config_dir = app.path().app_config_dir().map_err(|e| format!("{}", e))?;
            app.manage(SettingsPath(config_dir.join("settings.json")));

            // Start F# sidecar via dotnet run
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let backend_dir = manifest_dir.join("..").join("..").join("backend");
            let backend_dir = std::fs::canonicalize(&backend_dir).map_err(|e| format!("Cannot find backend dir: {}", e))?;
            log::info!("Starting F# sidecar from: {:?}", backend_dir);

            let dotnet_child = Command::new("dotnet")
                .args(["run", "--no-build"])
                .current_dir(&backend_dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to start F# sidecar: {}", e))?;

            *app.state::<Sidecar>().0.lock().unwrap() = Some(dotnet_child);
            log::info!("F# sidecar started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
