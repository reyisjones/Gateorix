# Security Model

Gateorix enforces manifest permissions for built-in host-core plugins. This is not an OS sandbox: native plugins, custom handlers and backend processes are trusted code running with the user's privileges. Separate example Tauri commands do not automatically inherit host-core checks.

## Core Principles

### 1. Deny by Default

All dangerous capabilities — filesystem access, process execution, clipboard, notifications — are **disabled** unless explicitly declared in the application manifest (`gateorix.config.json`).

Built-in plugin calls with an empty permissions block cannot read files, execute processes, or access the clipboard. This does not restrict arbitrary native code outside those plugins.

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

Create `Bridge::with_permissions(guard)` and attach built-in plugins using `register_plugin`. `Bridge::new()` denies requests in the filesystem, process, clipboard and notifications namespaces. Custom namespaces and low-level `register_handler` closures remain trusted integration code.

Migration: `Plugin::handle` now takes `&PermissionGuard` as a second argument. All four built-in implementations enforce it, including direct calls. `FilesystemPlugin::new()` no longer accepts a base directory; the guard owns the approved directory capabilities. Applications must create their allowed directories before constructing the guard. Invalid or unavailable scopes fail closed and require recreating the guard after correction.

### 3. Sandboxed Webview

The frontend runs in a native webview that:

- Cannot import Node.js modules.
- Cannot make direct system calls.
- Uses Tauri commands in desktop examples or direct loopback HTTP in browser development.
- Still requires appropriate application CSP, navigation and Tauri capability configuration.

### 4. Scoped Filesystem Access

Filesystem scopes and request paths must be project-relative. Absolute paths and parent components (`..`) are rejected. The guard opens existing allowed directories as `cap-std` directory capabilities. Built-in filesystem operations execute relative to those handles rather than checking a string and then using unrestricted filesystem APIs. Symlink resolution outside the capability is denied during the actual operation, including new-file writes.

Scopes are trusted manifest configuration; their initial directory resolution occurs during guard construction. Choose a trusted project root and protect scope configuration. Capability containment does not prevent access to an already-created hard link within a scope, nor isolate a malicious native plugin. Missing intermediate directories are not automatically created by file writes.

### 5. Mediated Sidecar Communication

Desktop integrations are intended to use:

```
Frontend → Bridge → Host Core → Runtime Adapter
```

Browser development instead uses direct loopback HTTP. Backend command authorization is the application's responsibility; host plugin permissions are not a backend sandbox. The host-core stdio `send()` implementation remains a placeholder and is not a complete production relay.

The Rust SDK development server binds to `127.0.0.1`, checks Host and browser Origin, requires JSON POSTs with bounded Content-Length, and limits messages to 1 MiB. It is unauthenticated, has no application-level request deadline, and is not suitable for hostile local clients or production deployment. Do not register privileged handlers on it without an additional security design. CORS does not authenticate local processes.

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
| Sidecar process escape | Not isolated; backend code is trusted |
| Malformed IPC messages | JSON validation + schema checks |
| Supply chain attacks in plugins | Native plugin code is trusted; review dependencies and provenance |

## Future Enhancements

- **Content Security Policy (CSP)** enforcement in the webview.
- **Plugin sandboxing** with WASM isolation.
- **Binary IPC validation** when binary transport is added.
- **Code signing** verification for sidecar binaries.
- **Network policy** controls for sidecar HTTP transport.
