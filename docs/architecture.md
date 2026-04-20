# Architecture

Gateorix is organized into six layers, each with clearly defined responsibilities. Communication between layers flows through well-defined interfaces and is always permission-checked.

## Layer Diagram

```
┌─────────────────────────────────────────────────────────┐
│                     Frontend Layer                      │
│            React / Vue / Svelte / HTML + JS             │
│                                                         │
│  Communicates exclusively through @gateorix/bridge      │
└────────────────────────┬────────────────────────────────┘
                         │
                         │  IPC (JSON over webview channel)
                         │
┌────────────────────────▼────────────────────────────────┐
│                     Bridge Layer                        │
│                                                         │
│  • Marshals requests between UI and Host Core / Runtime │
│  • Validates message structure                          │
│  • Enforces request/response correlation (id matching)  │
│  • Supports: request/response, events, streaming (v2)   │
└────────┬──────────────────────────────────┬─────────────┘
         │                                  │
┌────────▼──────────┐            ┌──────────▼─────────────┐
│   Host Core       │            │  Runtime Adapter Layer │
│                   │            │                        │
│  • App lifecycle  │            │  • Adapter protocol    │
│  • Window mgmt    │            │  • Sidecar process mgmt│
│  • Menu / tray    │◄──────────►│  • Python, Go, .NET,   │
│  • Permission     │  routing   │    Swift, Obj-C workers│
│    enforcement    │            │  • stdio / HTTP IPC    │
│  • Plugin host    │            │                        │
└────────┬──────────┘            └────────────────────────┘
         │
┌────────▼──────────┐
│   Plugin Layer    │
│                   │
│  • filesystem     │
│  • process        │
│  • clipboard      │
│  • notifications  │
│  • (custom)       │
└───────────────────┘

┌─────────────────────────────────────────────────────────┐
│                   Packaging Layer                       │
│                                                         │
│  • Build frontend assets (Vite)                         │
│  • Compile host core (Cargo)                            │
│  • Bundle sidecar binaries                              │
│  • Produce OS-native installers (dmg, msi, AppImage)    │
└─────────────────────────────────────────────────────────┘
```

## 1. Frontend Layer

The UI runs inside a platform-native webview. Any web framework can be used — React, Vue, Svelte, or plain HTML/JS. The frontend **never** accesses OS APIs directly; all interaction goes through the `@gateorix/bridge` SDK.

**Key constraint:** The webview is sandboxed. There is no `require('fs')`, no `child_process`, no direct network calls to the sidecar. Everything is mediated by the bridge.

## 2. Bridge Layer

The bridge is the secure boundary between the webview and the native host. It:

- Accepts `IpcRequest` messages from the frontend.
- Routes them to the correct handler (plugin or runtime adapter) based on the channel namespace.
- Returns `IpcResponse` messages to the frontend.
- Pushes `IpcEvent` messages for fire-and-forget notifications (e.g. runtime stdout, window close requested).

**Protocol:** JSON over the webview's native IPC channel (Tauri's invoke mechanism). Binary transport is planned for v2.

## 3. Host Core

The Rust-native heart of the framework. Responsibilities:

| Module | Purpose |
|---|---|
| `app` | Manifest loading, configuration, lifecycle |
| `window` | Window creation, resizing, close handling |
| `ipc` | Bridge dispatcher and protocol types |
| `permissions` | Capability checks against the manifest |
| `plugins` | Plugin registration and dispatch |
| `runtime` | Adapter lifecycle and message relay |

## 4. Runtime Adapter Layer

This is what makes Gateorix language-agnostic. The host core:

1. Reads the `runtime` section of the manifest.
2. Spawns a sidecar process for the chosen language (e.g. `python3 backend/main.py`).
3. Communicates over stdio (newline-delimited JSON) or local HTTP.
4. Relays messages between the frontend bridge and the sidecar.

The sidecar process uses a language-specific SDK (e.g. `gateorix` Python package) to register command handlers and run the message loop.

**Dual IPC modes:** In development, the frontend can fall back to an HTTP bridge (port 3001) for browser-based iteration without compiling the Rust host. In production (Tauri webview), all IPC goes through native invoke commands.

## 5. Application Shell Layer

The hello-react-python example demonstrates the canonical app shell pattern that serves as a template for generated projects:

| Component | Purpose |
|---|---|
| `Navbar` | Logo, theme toggle, profile button, login/logout |
| `HomePage` | Greet demo — end-to-end IPC round trip |
| `ProfilePage` | Display name and email, persisted to `settings.json` via Tauri |
| `LoginPage` | Form → Rust `login` command with hardcoded demo credentials |
| `Theme` | Dark/light toggle using CSS custom properties, persisted to settings |

**Settings storage:** A `settings.json` file in the platform config directory (`app.path().app_config_dir()`) stores theme preference, profile data, and other app state. The Rust `get_settings` and `save_settings` commands handle read/merge-write operations.

**Authentication:** The `login` command validates credentials on the Rust side and returns a display name. The `logout` command is a no-op on the backend since session state lives in the frontend.

## 5. Plugin Layer

Plugins expose OS capabilities to the bridge. Each plugin:

- Implements the `Plugin` trait.
- Registers under a namespace (e.g. `filesystem`, `clipboard`).
- Handles IPC requests on its namespace channels.
- Is subject to permission checks before execution.

Built-in plugins: `filesystem`, `process`, `notifications`, `clipboard`. Custom plugins can be added.

## 6. Packaging Layer

Handles building and distributing the final application:

- Frontend assets built with Vite (or any bundler).
- Host core compiled with Cargo.
- Sidecar binaries bundled alongside the app.
- OS-native installers produced (`.dmg` for macOS, `.msi` for Windows, `.AppImage`/`.deb` for Linux).

## Security Model

See [security.md](security.md) for the full security design. Key principles:

- **Deny by default** — all capabilities require explicit manifest grants.
- **No direct sidecar access** — the frontend never talks to the runtime adapter directly.
- **Scoped filesystem** — file access limited to declared paths.
- **IPC validation** — all messages validated before dispatch.
