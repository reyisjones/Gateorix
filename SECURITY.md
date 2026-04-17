# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.x.x   | ✅ Current development |

## Reporting a Vulnerability

If you discover a security vulnerability in Gateorix, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please send an email to **security@gateorix.dev** (or open a private security advisory on GitHub) with:

1. A description of the vulnerability.
2. Steps to reproduce.
3. The potential impact.
4. Any suggested fix (optional).

We will acknowledge receipt within 48 hours and aim to provide a fix or mitigation within 7 days for critical issues.

## Security Design Principles

Gateorix follows these security principles by design:

- **Deny by default** — All dangerous OS capabilities are disabled unless explicitly granted in the app manifest.
- **Permission-based access** — Filesystem, process execution, clipboard, and other OS features require manifest declarations.
- **IPC validation** — All messages crossing the bridge boundary are validated against expected schemas.
- **Sandboxed webview** — The frontend cannot directly access OS APIs; it must go through the bridge.
- **Scoped filesystem** — File access is restricted to paths declared in the manifest permissions.
- **No direct sidecar access** — The frontend never communicates directly with runtime adapters; the host core always mediates.
