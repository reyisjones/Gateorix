# Adapter Protocol

The adapter protocol defines how the Gateorix host core communicates with runtime adapter sidecar processes. It is designed to be language-agnostic — any language that can read stdin and write stdout (or serve HTTP) can implement an adapter.

## Overview

```
Host Core ──stdio──► Sidecar Process
           ◄──stdio──
```

Or with HTTP transport:

```
Host Core ──HTTP POST──► Sidecar HTTP Server
           ◄──response──
```

## Transport Options

### Stdio (Default)

The sidecar reads newline-delimited JSON from stdin and writes newline-delimited JSON to stdout. Each message is a single JSON object terminated by `\n`.

**Advantages:** Simple, no port conflicts, no network exposure.

### HTTP (Optional)

The sidecar starts a local HTTP server on a configured port. The host core sends POST requests with JSON bodies and reads JSON responses.

**Advantages:** Familiar for web developers, supports concurrent requests natively.

## Message Format

### Request (Host → Sidecar)

```json
{
  "id": "req-1001",
  "channel": "runtime.greet",
  "payload": {
    "name": "Alice"
  }
}
```

| Field | Type | Description |
|---|---|---|
| `id` | string | Unique request identifier for response correlation |
| `channel` | string | Target command in `runtime.<action>` format |
| `payload` | object | Arbitrary command-specific data |

### Response (Sidecar → Host)

Success:
```json
{
  "id": "req-1001",
  "ok": true,
  "payload": {
    "message": "Hello, Alice!"
  }
}
```

Error:
```json
{
  "id": "req-1001",
  "ok": false,
  "payload": {
    "error": "name is required"
  }
}
```

| Field | Type | Description |
|---|---|---|
| `id` | string | Must match the request ID |
| `ok` | boolean | Whether the command succeeded |
| `payload` | object | Result data on success, or `{ "error": "..." }` on failure |

## Lifecycle

1. **Spawn:** The host core spawns the sidecar process based on `gateorix.config.json`.
2. **Ready:** The sidecar starts its message loop (reading stdin or listening on HTTP).
3. **Communication:** The host core relays IPC requests from the bridge to the sidecar.
4. **Shutdown:** When the app closes, the host core sends a shutdown signal and terminates the process.

## Implementing an Adapter

### Python (using the SDK)

```python
from gateorix import GateorixAdapter

adapter = GateorixAdapter()

@adapter.command("greet")
def greet(payload):
    name = payload.get("name", "World")
    return {"message": f"Hello, {name}!"}

adapter.run()
```

### Any Language (raw protocol)

Read JSON lines from stdin, parse, process, and write JSON lines to stdout:

```
read line from stdin → parse JSON → find handler → execute → write JSON to stdout
```

The only requirement is that:
1. Each line is a valid JSON object.
2. The response `id` matches the request `id`.
3. The response contains `ok` (boolean) and `payload` (object).

## Future Enhancements

- **Binary transport** using MessagePack or Protocol Buffers.
- **Streaming** support for long-running operations.
- **Health checks** for monitoring sidecar liveness.
- **Multiplexed channels** for concurrent request processing.
