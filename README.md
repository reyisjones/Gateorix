<p align="center">
  <h1 align="center">Gateorix</h1>
  <p align="center"><strong>Web UI. Native power.</strong></p>
  <p align="center">
    A cross-platform desktop framework that pairs modern web frontends with native OS access — and lets you write backend logic in any language you prefer.
  </p>
</p>

<p align="center">
  <a href="https://github.com/reyisjones/Gateorix/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="https://github.com/reyisjones/Gateorix/actions"><img src="https://img.shields.io/github/actions/workflow/status/reyisjones/Gateorix/ci.yml?branch=main" alt="CI"></a>
  <a href="https://www.npmjs.com/package/@gateorixjs/cli"><img src="https://img.shields.io/npm/v/@gateorixjs/cli" alt="npm"></a>
</p>

---

## What is Gateorix?

Gateorix is a lightweight desktop application framework built on [Tauri](https://tauri.app) and Rust. It gives you:

- **A web-based frontend** — use React, Vue, Svelte, or plain HTML/JS.
- **A native host runtime** — window management, menus, system tray, file dialogs, notifications, and more.
- **Backend language adapters** — write your business logic in Python, Go, C#, F#, or any language that compiles to a binary.
- **A plugin system** — extend OS capabilities with first-party and custom plugins.
- **A secure IPC bridge** — all communication between frontend, host, and backend is permission-checked and sandboxed.
- **A VS Code extension** — IntelliSense, config validation, and integrated commands.

Think of it as the gateway between modern web UI and native desktop power.

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                  Frontend Layer                  │
│         (React / Vue / Svelte / HTML+JS)         │
└──────────────────────┬──────────────────────────┘
                       │  Gateorix Bridge API
                       │  (Tauri native or HTTP fallback)
┌──────────────────────▼──────────────────────────┐
│                  Bridge Layer                    │
│       (Secure IPC · JSON messages · events)      │
└──────┬───────────────────────────────┬──────────┘
       │                               │
┌──────▼──────────┐          ┌─────────▼──────────┐
│   Host Core     │          │  Runtime Adapters   │
│  (Rust/Tauri)   │          │  Python · Go · .NET │
│  Windows, menus │          │  Sidecar processes  │
│  Tray, dialogs  │          │  stdio / HTTP IPC   │
│  Logging, perms │          └────────────────────┘
└──────┬──────────┘
       │
┌──────▼──────────┐
│  Plugin Layer   │
│  FS · Process   │
│  Clipboard ·    │
│  Notifications  │
└─────────────────┘
```

## Quick Start

```bash
# Install the CLI
npm install -g @gateorixjs/cli

# Create a new project
gateorix init my-app --template react-python

# Start development
cd my-app
gateorix dev

# Build for production
gateorix build
```

## CLI Commands

| Command | Description |
|---|---|
| `gateorix init <name>` | Scaffold a new project from a template |
| `gateorix dev` | Start the app in development mode with hot reload |
| `gateorix build` | Build the app for production |
| `gateorix doctor` | Check environment and dependencies |
| `gateorix add runtime <lang>` | Add a runtime adapter (python, go, dotnet, swift) |
| `gateorix add plugin <name>` | Add a plugin (filesystem, process, notifications, clipboard) |

## Project Structure

```
gateorix/
├── cli/                     # CLI tool (TypeScript, published as @gateorixjs/cli)
├── host-core/               # Native host runtime (Rust)
│   └── src/logging.rs       # Structured logging (tracing + JSON)
├── sdk/                     # Language SDKs and bridge libraries
│   ├── js/                  # JavaScript/TypeScript bridge
│   ├── python/              # Python adapter SDK
│   ├── go/                  # Go adapter SDK (stdio + HTTP)
│   ├── dotnet/              # .NET adapter SDK (stdio + ASP.NET Minimal API)
│   └── swift/               # Swift adapter SDK (planned)
├── plugins/                 # Host plugins
│   ├── filesystem/
│   ├── process/
│   ├── notifications/
│   └── clipboard/
├── vscode-extension/        # VS Code extension (IntelliSense, commands, diagnostics)
├── templates/               # Starter templates
├── examples/                # Example applications
│   └── hello-react-python/  # Full demo: React + Python + Tauri
└── docs/                    # Documentation
```

## Supported Backend Languages

| Language | Adapter Status | IPC Method |
|---|---|---|
| Python | ✅ Implemented | stdio / HTTP |
| Go | ✅ Implemented | stdio / HTTP |
| C# / F# (.NET) | ✅ Implemented | stdio / HTTP |
| Swift | 🟡 Planned (Phase 3) | stdio |

## How It Works

1. **Frontend** renders in an embedded webview (or browser during dev) and communicates through the Gateorix bridge API. Supports dual IPC mode — Tauri native when running as a desktop app, HTTP fallback for browser-based development.
2. **Host Core** (Rust) manages the app lifecycle, windows, menus, system tray, file dialogs, and enforces the permission model. Includes structured logging with `tracing`.
3. **Runtime Adapters** spawn backend processes (sidecars) in your chosen language. The host core relays IPC messages between the frontend and these processes via stdio or HTTP.
4. **Plugins** expose OS capabilities (filesystem, clipboard, notifications) through a secure, permission-gated API.

All IPC uses JSON messages with request/response and event patterns. Binary transport (MessagePack / Protocol Buffers) is planned for Phase 4.

## Features

### Desktop Shell
- Window management — multi-window, resize, fullscreen, minimize, maximize
- Application menus — File, View, Help with keyboard shortcuts
- System tray — show/hide, quit
- File dialogs — open file, save file, open folder
- Dark / light theme toggle with disk persistence

### Developer Experience
- `gateorix dev` orchestrates frontend, backend, and Tauri processes
- `gateorix build` produces native installers via Tauri
- `gateorix doctor` validates your full toolchain and project config
- VS Code extension with IntelliSense for `gateorix.config.json`
- Structured logging with JSON output and env-based filtering

### CI/CD
- GitHub Actions CI for Rust (cross-platform) and CLI (TypeScript)
- Automated npm publishing when CLI version changes

## Documentation

- [Architecture](docs/architecture.md)
- [Security Model](docs/security.md)
- [Adapter Protocol](docs/adapter-protocol.md)
- [Roadmap](docs/roadmap.md)

## Current Status

**Phase 1 (Foundation)** and **Phase 2 (Developer Experience)** are complete. The framework includes a working CLI, Tauri integration, three runtime adapter SDKs (Python, Go, .NET), structured logging, and a VS Code extension.

**Phase 3 (Production Readiness)** is next — OS-native packaging, auto-update, code signing, and sidecar health monitoring.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute.

## Security

See [SECURITY.md](SECURITY.md) for our security policy and how to report vulnerabilities.

## License

Gateorix is released under the [MIT License](LICENSE).
