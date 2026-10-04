# @gateorixjs/cli

**Gateorix CLI** — scaffold, develop, and build cross-platform desktop apps with web UI and native power.

Gateorix pairs modern web frontends (React, Vue, Svelte) with native OS access through Rust/Tauri, and lets you write backend logic in any language you prefer.

## Install

```bash
npm install -g @gateorixjs/cli
```

This installs **two binaries** — `gateorix` and its short alias `gx`. They are identical; use whichever you prefer.

```bash
gateorix --version   # 0.3.2
gx --version         # 0.3.2 (same binary)
```

## Commands

| Command | Alias | Description |
|---|---|---|
| `gateorix init <name>` | `gx init <name>` | Scaffold a new project (interactive prompts for backend & UI) |
| `gateorix dev` | `gx dev` | Start the app in development mode with hot reload |
| `gateorix build` | `gx build` | Build the app for production |
| `gateorix doctor` | `gx doctor` | Check environment and dependencies |
| `gateorix add runtime <lang>` | `gx add runtime <lang>` | Add a runtime adapter |
| `gateorix add plugin <name>` | `gx add plugin <name>` | Add a plugin |

## Quick Start

```bash
# Interactive — prompts for project name, backend language, and UI framework
gx init my-app

cd my-app
gx dev
```

## Supported Templates (30 combinations in this checkout)

`gx init` prompts for a **UI framework** and a **backend language**. The 25 checked-in examples are supplemented by five generated Rust browser templates. Rust support is unreleased and requires a CLI built from this checkout.

**UI frameworks:** `react`, `vue`, `svelte`, `solid`, `vanilla`

**Backend languages:** `python`, `go`, `c#`, `f#`, `c++`, `rust`

Non-Rust combinations map to examples at `examples/hello-<ui>-<lang>` — for instance:

- `gx init my-app` → pick `vue` + `go` → scaffolds from `hello-vue-go`
- `gx init my-app` → pick `svelte` + `c#` → scaffolds from `hello-svelte-cs`
- `gx init my-app` → pick `vanilla` + `python` → scaffolds from `hello-vanilla-python`

## Adding Runtimes & Plugins

```bash
gx add runtime python     # or: go, dotnet, cpp, swift, rust
gx add plugin filesystem  # or: process, notifications, clipboard
```

## What You Get

- **5 UI frameworks** — React, Vue 3, Svelte, SolidJS, or plain HTML/TS (Vanilla)
- **6 backend choices** — Python, Go, C#, F#, C++, Rust
- **Native host runtime** — windows, menus, file access, notifications via Rust/Tauri
- **Plugin system** — filesystem, clipboard, notifications, process, and custom plugins
- **Secure IPC bridge** — JSON messages with permission-based access control

## Requirements

For Rust browser projects, use `gx init my-app --template react-rust --no-install`, install frontend dependencies, then run `gx dev`. Omit `--no-install` for automatic dependency installation. `gx build --release` compiles backend and frontend assets, not a native Rust installer. Adding Rust to an existing native shell is explicitly rejected.

Rust templates bundle SDK source and are prepared by `npm run sync-templates`. `npm test` tests the packed CLI outside the repository, including all five Rust selections and stdio/HTTP backend calls. These new options are not yet in the published 0.3.2 release.

- Node.js >= 18
- Rust/Cargo >= 1.82 for the Rust backend adapter (other native examples have their own requirements)
- Python 3.10+ (for Python backend adapter)

## Links

- [GitHub](https://github.com/reyisjones/Gateorix)
- [Architecture](https://github.com/reyisjones/Gateorix/blob/main/docs/architecture.md)
- [Roadmap](https://github.com/reyisjones/Gateorix/blob/main/docs/roadmap.md)

## License

MIT
