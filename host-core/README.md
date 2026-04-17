# Gateorix Host Core

The native host runtime for the Gateorix desktop framework, written in Rust.

## Modules

| Module | Responsibility |
|---|---|
| `app` | Application lifecycle, manifest loading, configuration |
| `window` | Window creation, management, and webview integration |
| `ipc` | Secure IPC bridge between frontend, host, and runtime adapters |
| `permissions` | Permission enforcement based on the app manifest |
| `plugins` | Plugin registration, discovery, and dispatch |
| `runtime` | Runtime adapter management — spawn, monitor, and communicate with sidecar processes |

## Building

```bash
cargo build -p gateorix-host-core
```

## Testing

```bash
cargo test -p gateorix-host-core
```
