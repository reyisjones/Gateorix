<p align="center">
  <h1 align="center">Gateorix</h1>
  <p align="center"><strong>Web UI. Native power.</strong></p>
  <p align="center">
    A cross-platform desktop framework that pairs modern web frontends with native OS access — and lets you write backend logic in any language you prefer.
  </p>
</p>

<p align="center">
  <a href="https://github.com/gateorix/gateorix/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="https://github.com/gateorix/gateorix/actions"><img src="https://img.shields.io/github/actions/workflow/status/gateorix/gateorix/ci.yml?branch=main" alt="CI"></a>
</p>

---

## What is Gateorix?

Gateorix is a lightweight desktop application framework built on [Tauri](https://tauri.app) and Rust. It gives you:

- **A web-based frontend** — use React, Vue, Svelte, or plain HTML/JS.
- **A native host runtime** — window management, menus, file access, notifications, and more.
- **Backend language adapters** — write your business logic in Python, Go, C#, F#, Swift, or any language that compiles to a binary or runs as a process.
- **A plugin system** — extend OS capabilities with first-party and custom plugins.
- **A secure IPC bridge** — all communication between frontend, host, and backend is permission-checked and sandboxed.

Think of it as the gateway between modern web UI and native desktop power.

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                  Frontend Layer                 │
│         (React / Vue / Svelte / HTML+JS)        │
└──────────────────────┬──────────────────────────┘
                       │  Gateorix Bridge API
┌──────────────────────▼──────────────────────────┐
│                  Bridge Layer                   │
│       (Secure IPC · JSON messages · events)     │
└──────┬───────────────────────────────┬──────────┘
       │                               │
┌──────▼──────────┐          ┌─────────▼──────────┐
│   Host Core     │          │  Runtime Adapters  │
│  (Rust/Tauri)   │          │  (Python, Go, C#,  │
│  Windows, menus │          │   F#, Swift, …)    │
│  FS, tray, etc. │          │  Sidecar processes │
└──────┬──────────┘          └────────────────────┘
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
npm install -g @gateorix/cli

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
├── cli/                  # CLI tool (TypeScript)
├── host-core/            # Native host runtime (Rust)
├── sdk/                  # Language SDKs and bridge libraries
│   ├── js/               # JavaScript/TypeScript bridge
│   ├── python/           # Python adapter SDK
│   ├── dotnet/           # .NET adapter SDK
│   ├── go/               # Go adapter SDK
│   └── swift/            # Swift adapter SDK
├── plugins/              # Host plugins
│   ├── filesystem/
│   ├── process/
│   ├── notifications/
│   └── clipboard/
├── templates/            # Starter templates
├── examples/             # Example applications
└── docs/                 # Documentation
```

## Supported Backend Languages

| Language | Adapter Status | IPC Method |
|---|---|---|
| Python | 🟢 v1 | stdio / HTTP |
| Go | 🟡 Planned | stdio / HTTP |
| C# / F# (.NET) | 🟡 Planned | stdio / HTTP |
| Swift | 🟡 Planned | stdio |
| Objective-C | 🟡 Planned | stdio |

## How It Works

1. **Frontend** renders in an embedded webview and communicates exclusively through the Gateorix bridge API.
2. **Host Core** (Rust) manages the app lifecycle, windows, menus, system tray, and enforces the permission model.
3. **Runtime Adapters** spawn backend processes (sidecars) in your chosen language. The host core relays IPC messages between the frontend and these processes.
4. **Plugins** expose OS capabilities (filesystem, clipboard, notifications) through a secure, permission-gated API.

All IPC uses JSON messages with request/response and event patterns. Binary transport is planned for a future release.

## Documentation

- [Architecture](docs/architecture.md)
- [Security Model](docs/security.md)
- [Adapter Protocol](docs/adapter-protocol.md)
- [Roadmap](docs/roadmap.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute.

## Security

See [SECURITY.md](SECURITY.md) for our security policy and how to report vulnerabilities.

## License

Gateorix is released under the [MIT License](LICENSE).
