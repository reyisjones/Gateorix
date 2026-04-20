# @gateorixjs/cli

**Gateorix CLI** — scaffold, develop, and build cross-platform desktop apps with web UI and native power.

Gateorix pairs modern web frontends (React, Vue, Svelte) with native OS access through Rust/Tauri, and lets you write backend logic in any language you prefer.

## Install

```bash
npm install -g @gateorixjs/cli
```

## Commands

```bash
gateorix init <name>            # Scaffold a new project from a template
gateorix dev                    # Start the app in development mode
gateorix build                  # Build the app for production
gateorix doctor                 # Check environment and dependencies
gateorix add runtime <lang>     # Add a runtime adapter (python, go, dotnet, swift)
gateorix add plugin <name>      # Add a plugin (filesystem, process, notifications, clipboard)
```

## Quick Start

```bash
gateorix init my-app --template react-python
cd my-app
gateorix dev
```

## What You Get

- **Web-based frontend** — React, Vue, Svelte, or plain HTML/JS
- **Native host runtime** — windows, menus, file access, notifications via Rust/Tauri
- **Backend language adapters** — Python, Go, C#, Swift, and more
- **Plugin system** — filesystem, clipboard, notifications, process, and custom plugins
- **Secure IPC bridge** — JSON messages with permission-based access control

## Requirements

- Node.js >= 18
- Rust (for Tauri compilation)
- Python 3.10+ (for Python backend adapter)

## Links

- [GitHub](https://github.com/reyisjones/Gateorix)
- [Architecture](https://github.com/reyisjones/Gateorix/blob/main/docs/architecture.md)
- [Roadmap](https://github.com/reyisjones/Gateorix/blob/main/docs/roadmap.md)

## License

MIT
