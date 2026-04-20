# Roadmap

Gateorix development is organized into phases. Each phase builds on the previous one, adding capabilities incrementally while keeping the framework stable and usable at each stage.

---

## Phase 1 — Foundation (Current)

**Goal:** Establish the core architecture, project scaffold, and first working vertical slice.

- [x] Project structure and open-source setup (README, LICENSE, CONTRIBUTING, etc.)
- [x] Rust workspace with host-core crate
- [x] IPC protocol and bridge dispatcher
- [x] Permission system with manifest-based enforcement
- [x] Plugin trait and registry
- [x] Runtime adapter trait and stdio process manager
- [x] CLI scaffold (TypeScript) with init, dev, build, doctor commands
- [x] JavaScript bridge SDK (`@gateorix/bridge`)
- [x] Python adapter SDK (`gateorix`)
- [x] Filesystem, process, notifications, clipboard plugin stubs
- [x] Starter templates (react-python, vanilla)
- [x] Documentation: architecture, security model, adapter protocol
- [x] Integration: wire host core to a real Tauri webview
- [x] End-to-end: frontend → bridge → host → Python sidecar round trip

---

## Phase 2 — Developer Experience

**Goal:** Make the framework usable for real projects with a smooth development workflow.

- [x] HTTP dev bridge for browser-based development (Python backend, port 3001)
- [x] Tauri webview integration with native IPC (invoke_backend command)
- [x] App shell template — navbar, page routing, responsive layout
- [x] Dark / light theme toggle with disk persistence (settings.json)
- [x] Profile page — display name and email saved via Tauri commands
- [x] Login / logout flow with Rust-side credential validation
- [x] Dual IPC mode — Tauri native or HTTP bridge fallback
- [x] CSS custom properties for theming
- [ ] `gateorix dev` — start frontend + host + sidecar with hot reload
- [ ] `gateorix build` — produce a bundled app with sidecar
- [ ] `gateorix doctor` — full environment checks
- [ ] Window management (multi-window, resize, fullscreen)
- [ ] Menu and system tray support
- [ ] File dialog integration
- [ ] Go runtime adapter SDK
- [ ] .NET (C#/F#) runtime adapter SDK
- [ ] Structured logging and diagnostics
- [ ] VS Code extension for debugging and IntelliSense

---

## Phase 3 — Production Readiness

**Goal:** Ship stable, secure, installable desktop applications.

- [ ] OS-native packaging (`.dmg`, `.msi`, `.AppImage`, `.deb`)
- [ ] Auto-update mechanism
- [ ] Code signing for macOS and Windows
- [ ] Content Security Policy (CSP) in the webview
- [ ] Sidecar health monitoring and restart
- [ ] Swift runtime adapter SDK
- [ ] HTTP transport for runtime adapters
- [ ] Plugin: deep links / URL scheme handling
- [ ] Plugin: system theme detection (dark/light mode)

---

## Phase 4 — Advanced Features

**Goal:** Enable complex, high-performance applications.

- [ ] Binary IPC transport (MessagePack / Protocol Buffers)
- [ ] Streaming messages for real-time data
- [ ] WASM plugin sandboxing
- [ ] Multi-window IPC routing
- [ ] Background workers
- [ ] Plugin marketplace / registry
- [ ] CI/CD templates for automated desktop app builds
- [ ] Cloud-connected plugin patterns (auth, storage, sync)

---

## Phase 5 — Ecosystem

**Goal:** Grow the developer community and ecosystem.

- [ ] Official website and documentation site
- [ ] Template gallery (React, Vue, Svelte, Angular, SolidJS)
- [ ] Community plugin registry
- [ ] Tutorials and video walkthroughs
- [ ] Performance benchmarks vs Electron, Tauri, Wails
- [ ] Conference talks and blog posts
