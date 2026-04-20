# Hello Gateorix — React + C#

A complete desktop application example using a **React** frontend with a **C#** (.NET 8) backend, powered by the Gateorix framework and Tauri v2.

## What It Does

- **Greet Demo** — Type a name and receive a greeting from the C# backend via IPC
- **Dark/Light Theme** — Toggle with settings persisted to disk
- **Profile Page** — Edit and save profile data (Tauri native mode)
- **Login Page** — Demo authentication (`admin / gateorix` or `demo / demo`)
- **Native Menus & Tray** — File menu, View menu, system tray icon

## Architecture

```
┌──────────────┐   JSON/stdio    ┌──────────────┐
│  Tauri Shell │ ◄────────────► │  C# Backend   │
│  (Rust/WRY)  │   (sidecar)    │  (.NET 8)     │
└──────┬───────┘                 └──────────────┘
       │ IPC
┌──────┴───────┐
│  React App   │
│  (Vite + TS) │
└──────────────┘
```

**Production**: Tauri spawns `dotnet run` as a sidecar process; communication via JSON over stdin/stdout.  
**Development**: C# runs an ASP.NET Minimal API HTTP bridge on `localhost:3001`; React dev server on `localhost:5173`.

## Prerequisites

| Tool      | Version   | Install                                    |
|-----------|-----------|--------------------------------------------|
| Node.js   | ≥ 18      | https://nodejs.org                         |
| .NET SDK  | 8.0       | https://dotnet.microsoft.com/download      |
| Rust      | ≥ 1.77    | https://rustup.rs                          |
| Tauri CLI | v2        | `npm install -g @tauri-apps/cli`           |

**Windows**: Also need [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" workload.

> **Note**: A `global.json` file pins the SDK to version 8.0.x. If you have multiple .NET SDKs installed, this ensures the correct one is used.

## Quick Start (Dev Mode)

### 1. Start the C# backend

```bash
cd backend
dotnet run -- --http
```

You should see:
```
  Gateorix HTTP dev bridge (C#)
  → listening on http://localhost:3001/invoke
  → CORS allowed: http://localhost:5173
```

### 2. Start the React frontend

```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173` in your browser. Type a name and click **Greet** — the greeting comes from C#.

### 3. Run as a native desktop app (optional)

```bash
cd frontend
npm run tauri dev
```

This builds the Tauri shell, spawns the C# sidecar via stdio, and opens a native window.

## How IPC Works

### Stdio (Production)

The Tauri Rust shell spawns `dotnet run --no-build` as a child process. Every command is a JSON line written to stdin:

```json
{"id":"abc-123","channel":"runtime.greet","payload":{"name":"Alice"}}
```

The C# backend reads the line, dispatches to the handler, and writes a JSON response to stdout:

```json
{"id":"abc-123","ok":true,"payload":{"message":"Hello from C#, Alice! Welcome to Gateorix."}}
```

### HTTP (Development)

With `--http`, C# starts an ASP.NET Minimal API server using `WebApplication`. The React app posts to `http://localhost:3001/invoke` with the same JSON body. CORS middleware allows requests from the Vite dev server at `localhost:5173`.

## Project Structure

```
hello-solid-cs/
├── gateorix.config.json      # Gateorix project manifest
├── backend/
│   ├── Program.cs             # C# backend (greet + echo handlers)
│   ├── HelloGateorixCs.csproj # .NET project file (SDK.Web, net8.0)
│   └── global.json            # SDK version pin
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
            └── lib.rs         # Tauri commands + dotnet sidecar spawn
```

## Adding a New Backend Command

1. Add a handler method in `backend/Program.cs`:
   ```csharp
   static object HandleMyCommand(JsonElement payload)
   {
       return new { result = "done" };
   }
   ```

2. Register it in the `handlers` dictionary:
   ```csharp
   ["mycommand"] = HandleMyCommand,
   ```

3. Call it from the React frontend:
   ```typescript
   const data = await invokeBackend("runtime.mycommand", { key: "value" });
   ```

## License

MIT — see [LICENSE](../../LICENSE) in the repository root.
