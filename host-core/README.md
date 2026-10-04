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

## Stdio Runtime Transport

`StdioProcess` implements the synchronous `RuntimeAdapter` interface using a
dedicated I/O worker per sidecar. It sends compact newline-delimited JSON and
retains its buffered reader across calls. Requests require a nonempty string
ID and channel plus an object payload. Responses require a matching ID, boolean
`ok`, and a `payload` field. Application errors (`ok: false`) remain valid
responses; invalid envelopes are transport errors.

- Frames are limited to 1 MiB, excluding the newline delimiter.
- A fixed 30-second deadline covers both stdin writes and response reads.
- Only one request may be in flight; concurrent calls fail immediately.
- Timeouts, malformed responses, wrong IDs and closed pipes invalidate the
	session and terminate the direct child. Start or restart before sending again.
- Stderr is continuously drained and discarded without accumulating output.
- Stop, restart and drop wait for the I/O worker and terminate/reap the direct
	child. Process status is monitored while idle as well as during requests.

`Running` means that spawning succeeded, not that a readiness handshake passed.
This adapter rejects HTTP transport. It does not multiplex events or responses,
provide a backend sandbox, or guarantee cleanup of descendants spawned by a
wrapper such as Cargo. Prefer a directly launched executable when child lifetime
control matters. Runtime-specific launcher improvements and integration with the
examples' separate Tauri bridges remain follow-up work.

Focused regression tests compile a small Rust fixture with `rustc`; no Python
interpreter or shell is required:

```bash
cargo test -p gateorix-host-core runtime::process::tests
```

See the [adapter protocol](../docs/adapter-protocol.md) and
[security model](../docs/security.md) for the wider contract and trust boundaries.
