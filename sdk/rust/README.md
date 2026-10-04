# Gateorix Rust Adapter

The local `gateorix-adapter` crate supports newline-delimited JSON over stdio and loopback HTTP for browser development. It is not published to crates.io; generated CLI templates include its source as a path dependency.

Requires Rust/Cargo 1.82 or newer. Register handlers with `GateorixAdapter::command`, then call `run()` for stdio or `run_http(3001)` for development HTTP. Requests contain a nonempty string `id`, string `channel` and object `payload`. Responses preserve the ID and contain `ok` plus `payload`. The adapter accepts bare command names or the `runtime.` prefix, rejects other dotted namespaces, and caps input frames at 1 MiB.

```rust
use gateorix_adapter::GateorixAdapter;

fn main() -> std::io::Result<()> {
    let mut adapter = GateorixAdapter::new();
    adapter.command("echo", Ok);
    adapter.run()
}
```

From a CLI built from this checkout:

```bash
gx init my-rust-app --template vanilla-rust
cd my-rust-app
gx dev
```

For stdio, run `cargo run --quiet --manifest-path backend/Cargo.toml` and send one JSON request per line. For an optimized backend executable, run `cargo build --release --manifest-path backend/Cargo.toml`. `gx build --release` also builds frontend assets. Native installer bundling is not configured for Rust templates.

The HTTP server binds to IPv4 loopback, accepts browser origin `http://localhost:5173`, validates Host, and exposes `POST /invoke` and `GET /health`. It has no authentication or application request deadline: use only trusted local development handlers. A Rust panic in a handler is not converted to a protocol error; handlers should return `Err(String)` for expected failures. Do not print logging to stdout in stdio mode.

Tests: `cargo test -p gateorix-adapter` from the repository root, and `npm test` inside the CLI directory for packed-consumer coverage.