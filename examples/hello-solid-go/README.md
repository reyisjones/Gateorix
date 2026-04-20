# Hello Gateorix — React + Go

A complete desktop application example using a **React** frontend with a **Go** backend, powered by the Gateorix framework and Tauri v2.

## What It Does

- **Greet Demo** — Type a name and receive a greeting from the Go backend via IPC
- **Dark/Light Theme** — Toggle with settings persisted to disk
- **Profile Page** — Edit and save profile data (Tauri native mode)
- **Login Page** — Demo authentication (`admin / gateorix` or `demo / demo`)
- **Native Menus & Tray** — File menu, View menu, system tray icon

## Architecture

```
┌──────────────┐   JSON/stdio    ┌──────────────┐
│  Tauri Shell  │ ◄────────────► │  Go Backend   │
│  (Rust/WRY)  │   (sidecar)    │  (main.go)    │
└──────┬───────┘                 └──────────────┘
       │ IPC
┌──────┴───────┐
│  React App   │
│  (Vite + TS) │
└──────────────┘
```

**Production**: Tauri spawns Go as a sidecar process; communication via JSON over stdin/stdout.  
**Development**: Go runs an HTTP bridge on `localhost:3001`; React dev server on `localhost:5173`.

## Prerequisites

| Tool      | Version | Install                                |
|-----------|---------|----------------------------------------|
| Node.js   | ≥ 18    | https://nodejs.org                     |
| Go        | ≥ 1.21  | https://go.dev/dl                      |
| Rust      | ≥ 1.77  | https://rustup.rs                      |
| Tauri CLI | v2      | `npm install -g @tauri-apps/cli`       |

**Windows**: Also need [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" workload.

## Quick Start (Dev Mode)

### 1. Start the Go backend

```bash
cd backend
go run main.go --http
```

You should see:
```
  Gateorix HTTP dev bridge (Go)
  → listening on http://localhost:3001/invoke
  → CORS allowed: http://localhost:5173
```

### 2. Start the React frontend

```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173` in your browser. Type a name and click **Greet** — the greeting comes from Go.

### 3. Run as a native desktop app (optional)

```bash
cd frontend
npm run tauri dev
```

This builds the Tauri shell, spawns the Go sidecar via stdio, and opens a native window.

## How IPC Works

### Stdio (Production)

The Tauri Rust shell spawns `go run main.go` as a child process. Every command is a JSON line written to stdin:

```json
{"id":"abc-123","channel":"runtime.greet","payload":{"name":"Alice"}}
```

The Go backend reads the line, dispatches to the handler, and writes a JSON response to stdout:

```json
{"id":"abc-123","ok":true,"payload":{"message":"Hello from Go, Alice! Welcome to Gateorix."}}
```

### HTTP (Development)

With `--http`, Go starts a minimal HTTP server. The React app posts to `http://localhost:3001/invoke` with the same JSON body. CORS headers allow requests from the Vite dev server at `localhost:5173`.

## Project Structure

```
hello-solid-go/
├── gateorix.config.json      # Gateorix project manifest
├── backend/
│   ├── main.go               # Go backend (greet + echo handlers)
│   └── go.mod                # Go module definition
└── frontend/
    ├── index.html
    ├── package.json
    ├── src/
    │   ├── main.tsx           # React app (Navbar, Home, Profile, Login)
    │   └── styles.css         # Dark/light theme CSS
    └── src-tauri/
        ├── Cargo.toml         # Rust dependencies
        ├── tauri.conf.json    # Tauri configuration
        ├── capabilities/      # Permission declarations
        └── src/
            └── lib.rs         # Tauri commands + Go sidecar spawn
```

## Adding a New Backend Command

1. Add a handler function in `backend/main.go`:
   ```go
   func handleMyCommand(payload json.RawMessage) (interface{}, error) {
       return map[string]string{"result": "done"}, nil
   }
   ```

2. Register it in the `handlers` map:
   ```go
   "mycommand": handleMyCommand,
   ```

3. Call it from the React frontend:
   ```typescript
   const data = await invokeBackend("runtime.mycommand", { key: "value" });
   ```

## License

MIT — see [LICENSE](../../LICENSE) in the repository root.
