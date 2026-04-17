# Security Model

Gateorix is designed with a defense-in-depth security model. Every layer enforces boundaries, and no single component has unrestricted access.

## Core Principles

### 1. Deny by Default

All dangerous capabilities — filesystem access, process execution, clipboard, notifications — are **disabled** unless explicitly declared in the application manifest (`gateorix.config.json`).

An application with an empty permissions block cannot read files, execute processes, or access the clipboard.

### 2. Manifest-Based Permissions

Permissions are declared statically in `gateorix.config.json`:

```json
{
  "permissions": {
    "filesystem": ["./data", "./config"],
    "process": false,
    "notifications": true,
    "clipboard": false
  }
}
```

The host core's `PermissionGuard` checks every IPC request against these declarations before dispatching to a plugin.

### 3. Sandboxed Webview

The frontend runs in a native webview that:

- Cannot import Node.js modules.
- Cannot make direct system calls.
- Cannot communicate with the sidecar process directly.
- Can only send messages through the Gateorix bridge API.

### 4. Scoped Filesystem Access

Filesystem operations are restricted to paths declared in the manifest. The `PermissionGuard` resolves relative paths against the project root and rejects any access outside the allowed scopes.

Path traversal attacks (e.g. `../../etc/passwd`) are prevented by canonicalizing paths before checking scope membership.

### 5. Mediated Sidecar Communication

The frontend **never** talks to the runtime adapter (sidecar) directly. All messages flow through:

```
Frontend → Bridge → Host Core → Runtime Adapter
```

This ensures every message passes through permission checks and validation before reaching the backend.

### 6. IPC Message Validation

All messages crossing the bridge boundary are:

- Parsed as JSON with strict deserialization.
- Checked for required fields (`id`, `channel`, `payload`).
- Routed only to registered handlers.
- Rejected with an error response if malformed.

## Threat Model

| Threat | Mitigation |
|---|---|
| Malicious frontend code accessing OS | Webview sandbox + bridge-only communication |
| Path traversal in filesystem plugin | Scoped path resolution in PermissionGuard |
| Unauthorized process execution | Process capability denied by default |
| Sidecar process escape | Host core mediates all communication |
| Malformed IPC messages | JSON validation + schema checks |
| Supply chain attacks in plugins | Plugin registry with namespace isolation |

## Future Enhancements

- **Content Security Policy (CSP)** enforcement in the webview.
- **Plugin sandboxing** with WASM isolation.
- **Binary IPC validation** when binary transport is added.
- **Code signing** verification for sidecar binaries.
- **Network policy** controls for sidecar HTTP transport.
