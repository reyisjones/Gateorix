You are Claude, an expert in full-stack development, Tauri desktop apps, and VS Code extension creation. Follow this **step-by-step guide** precisely to implement a complete end-to-end flow for a Tauri desktop app with an HTTP dev bridge and VS Code extension generator. Proceed **gradually**: Phase 1 (HTTP bridge + basic flow), Phase 2 (Tauri webview), Phase 3 (VS Code extension). Document every step in a clear Markdown structure with code snippets, commands, and explanations. Make it reusable as a template for other processes—include prerequisites, troubleshooting, and verification tests.

## Prerequisites
- Rust (1.80+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Node.js (20+), npm/yarn/pnpm
- Tauri CLI: `npm install -g @tauri-apps/cli`
- VS Code with extensions: Rust Analyzer, Tauri, vscode-webview-ui-toolkit
- Git repo initialized for the project (e.g., `my-tauri-app`). [github](https://github.com/tauri-apps/create-tauri-app)

## Phase 1: HTTP Dev Bridge (End-to-End Flow)
Create a simple HTTP server as a "dev bridge" for backend-frontend communication during development (mimics production IPC).

1. **Init Backend (Rust)**:  
   ```
   cargo new tauri-bridge --bin
   cd tauri-bridge
   cargo add axum tokio --features full warp serde json
   ```
   Create `src/main.rs`:
   ```rust
   use axum::{routing::get, Router, Json};
   use serde_json::json;
   #[tokio::main] async fn main() { /* ... */ } // See full code below
   ```
   Full server (exposes /api/echo, /health on localhost:3000):  
   ```rust
   use axum::{routing::get, Router, Json, response::Html};
   use serde_json::{json, Value};
   async fn health() -> Json<Value> { Json(json!({"status": "ok"})) }
   async fn echo(Json(payload): Json<Value>) -> Json<Value> { Json(payload) }
   #[tokio::main] async fn main() {
       let app = Router::new().route("/health", get(health)).route("/api/echo", get(echo).post(echo));
       let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
       axum::serve(listener, app).await.unwrap();
   }
   ```
   Run: `cargo run`. Test: `curl http://localhost:3000/health` → `{"status":"ok"}`. [jonaskruckenberg.github](https://jonaskruckenberg.github.io/tauri-docs-wip/development/windows-and-webviews.html)

2. **Frontend Web App**:  
   ```
   npm create vite@latest frontend -- --template vanilla-ts
   cd frontend
   npm i axios
   ```
   In `main.ts`, fetch from bridge:
   ```ts
   async function testBridge() {
     const res = await axios.get('http://localhost:3000/health');
     console.log(res.data); // {status: 'ok'}
   }
   ```
   Run dev server: `npm run dev` (localhost:5173). Verify end-to-end: Frontend calls backend via HTTP. [jonaskruckenberg.github](https://jonaskruckenberg.github.io/tauri-docs-wip/development/windows-and-webviews.html)

3. **Verification**: CORS enabled (`axum::middleware`), logs for requests. Commit: `git add . && git commit -m "Phase 1: HTTP Bridge"`.

## Phase 2: Tauri Webview Integration
Integrate the web app into Tauri, replacing HTTP bridge with Tauri's secure IPC.

1. **Scaffold Tauri**: From project root (with frontend subdir):  
   `npm create tauri-app --template vanilla --manager npm` (or integrate into existing). [github](https://github.com/tauri-apps/create-tauri-app)
   Edit `src-tauri/tauri.conf.json`: Set `devUrl: "http://localhost:5173"`, bundle static assets for prod.

2. **IPC Glue (Rust Commands)**: In `src-tauri/src/main.rs`:
   ```rust
   tauri::Builder::default()
     .invoke_handler(tauri::generate_handler![echo_cmd])
     .run(tauri::generate_context!())
     .expect("error");
   #[tauri::command] fn echo_cmd(input: String) -> String { input }
   ```

3. **Frontend Tauri Glue (JS/TS)**: Install `@tauri-apps/api`: `npm i @tauri-apps/api`.
   ```ts
   import { invoke } from '@tauri-apps/api/tauri';
   async function tauriEcho(input: string) { return await invoke('echo_cmd', { input }); }
   ```
   Update UI to use `tauriEcho` instead of axios.

4. **Dev/Prod Flow**: `npm run tauri dev` (uses webview on dev server). `npm run tauri build` (embeds static files). [v2.tauri](https://v2.tauri.app)
5. **Verification**: Test IPC: UI input → Rust command → response. No HTTP needed in prod. Commit: `git commit -m "Phase 2: Tauri Webview"`.

## Phase 3: VS Code Extension for Tauri Scaffold
Create a reusable VS Code extension that generates a full Tauri app with user-selected backend language (default Rust) and web framework (e.g., Vite/Vanilla, React, Svelte).

1. **Extension Setup**:
   ```
   npm install -g @vscode/vsce
   mkdir tauri-scaffolder && cd tauri-scaffolder
   npm init -y && npm i @tauri-apps/cli
   vsce package # Later
   ```
   `package.json`:
   ```json
   {
     "name": "tauri-scaffolder",
     "displayName": "Tauri App Scaffolder",
     "activationEvents": ["onCommand:tauri.scaffold"],
     "contributes": { "commands": [{ "command": "tauri.scaffold", "title": "Scaffold Tauri App" }] }
   }
   ```

2. **Webview UI for Inputs**: Use `vscode-window` for form: App name, backend lang (Rust/Python/Go), UI framework (Vanilla/React/Svelte/Vue), features (HTTP bridge, IPC glue).
   In `extension.ts`:
   ```ts
   import * as vscode from 'vscode';
   export function activate(context: vscode.ExtensionContext) {
     let disposable = vscode.commands.registerCommand('tauri.scaffold', async () => {
       const name = await vscode.window.showInputBox({ prompt: 'App name' });
       // Call shell: create-tauri-app, customize tauri.conf.json, generate glue
       vscode.commands.executeCommand('workbench.action.terminal.new');
       // Exec: npx create-tauri-app ${name} --template ${uiFramework}
     });
     context.subscriptions.push(disposable);
   }
   ```

3. **Auto-Generate Glue**: On scaffold:
   - Copy Phase 1/2 templates.
   - For Python backend: Use `tauri-plugin-python` or subprocess IPC. [v2.tauri](https://v2.tauri.app/about/philosophy/)
   - Install deps, `npm run tauri dev`.

4. **Package & Test**: `vsce package`, install .vsix in VS Code. Verify: Run command → scaffolds runnable Tauri app.
5. **Docs**: Extension README.md with screenshots, customization vars. Commit: `git commit -m "Phase 3: VS Code Extension"`.

## Final Deliverables
- Full project code/tree.
- Reusable template Markdown (this prompt).
- Tests: `cargo test`, `npm test`, manual IPC/UI flows.
- Troubleshooting: Port conflicts (kill 3000/5173), Rust paths (`rustup default stable`). [github](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md)

Output the complete implementation code, folder structure, and next steps. Keep responses concise per phase—confirm before advancing.