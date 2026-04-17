# Gateorix Starter Kit

## Product direction

Gateorix is a cross-platform desktop framework that combines a web UI with native operating system access through a lightweight host runtime and a language-adapter model. The goal is to offer a developer experience similar to modern frontend tooling while allowing backend logic in languages such as Python, Go, C#, F#, Swift, and Objective-C.

---

## Core vision

* Web UI for the frontend using React, Vue, plain HTML, or other web frameworks.
* Native host runtime for app lifecycle, window management, file access, process execution, notifications, and secure IPC.
* Multi-language backend adapters so developers can choose their preferred runtime.
* Plugin model for OS capabilities and custom extensions.
* Small, fast, developer-friendly alternative to heavier desktop frameworks.

---

## Positioning

Gateorix is the gateway between modern web UI and native desktop power.

Suggested tagline:
**Gateorix — Web UI. Native power.**

Alternative taglines:

* The gateway between web and system.
* Build desktop apps with web speed and native reach.
* Modern UI, real OS access.

---

## Architecture design prompt

Use this prompt in ChatGPT, VS Code Copilot, or another coding assistant to start the framework design.

```text
Design a cross-platform desktop application framework named Gateorix.

Goal:
Create a lightweight framework that allows developers to build desktop apps using a web-based frontend and a native host layer with access to operating system features. The framework must support multiple backend language adapters such as Python, Go, C#, F#, Swift, and Objective-C.

Main requirements:
1. Frontend
- Use an embedded webview for rendering the UI.
- Support React, Vue, or plain HTML/JavaScript.
- Provide hot reload in development mode.

2. Native host
- Manage windows, menus, dialogs, notifications, tray, file system access, and process execution.
- Expose OS capabilities through a secure API surface.
- Run on Windows, macOS, and Linux.

3. IPC bridge
- Create a secure bridge between frontend and native host.
- Support request/response, events, and streaming messages.
- Add permission-based capability checks.
- Use JSON message contracts initially, but design for binary transport later.

4. Multi-language runtime adapters
- Define a language-agnostic adapter protocol.
- Allow backend services to run in Python, Go, C#, F#, Swift, and Objective-C.
- Support starting, monitoring, and communicating with external runtimes.
- Allow packaging an app with one or more adapters.

5. Security
- Deny dangerous capabilities by default.
- Add manifest-based permissions.
- Validate all IPC messages.
- Restrict process execution and filesystem access to approved scopes.

6. Developer experience
- Provide a CLI with commands such as:
  gateorix init
  gateorix dev
  gateorix build
  gateorix doctor
  gateorix add runtime python
  gateorix add plugin fs
- Create starter templates.
- Include structured logging and diagnostics.

7. Packaging
- Support development and production builds.
- Package apps into native distributables per OS.
- Allow bundling external runtimes when needed.

8. Plugin system
- Create a plugin architecture for host features.
- Make it possible to add plugins like filesystem, shell, process, notifications, clipboard, and custom plugins.

9. Deliverables
- Propose a clean architecture.
- Define the main modules.
- Define the adapter protocol.
- Provide a folder structure.
- Provide initial interfaces and contracts.
- Provide a sample hello-world app using React frontend and Python backend.
- Suggest future enhancements such as binary IPC, sandboxing, and cloud-connected plugins.

Output format:
- High-level architecture
- Module responsibilities
- Folder structure
- Core interfaces
- Example IPC contract
- Sample CLI commands
- Roadmap with phases
```

---

## Recommended architecture

### Layers

1. **Frontend layer**

   * React, Vue, or HTML/JS
   * Talks only to the Gateorix bridge API

2. **Bridge layer**

   * Secure IPC boundary
   * Marshals requests from UI to host/runtime
   * Handles events and callbacks

3. **Host core**

   * Native app shell
   * Window lifecycle
   * Menus, tray, dialogs, notifications
   * Filesystem and process abstractions
   * Permissions enforcement

4. **Runtime adapter layer**

   * Adapter SDK and runtime contracts
   * Launch and monitor Python, Go, C#, F#, Swift, Objective-C workers
   * Route commands/results between host and external runtime

5. **Plugin layer**

   * Filesystem plugin
   * Shell/process plugin
   * Clipboard plugin
   * Notification plugin
   * Custom app plugins

6. **Packaging layer**

   * Build assets
   * Bundle runtime dependencies
   * Produce OS installers or app bundles

---

## Technical suggestion for v1

To reduce complexity, start with this stack for the first implementation:

* Host core: Rust or C++
* Webview: platform-native WebView abstraction
* CLI: Node.js/TypeScript or Rust
* Frontend starter: React + Vite
* First runtime adapter: Python
* IPC: local JSON over stdio or named pipes

Why this v1:

* Rust or C++ gives a strong native core.
* Python proves the multi-language runtime concept quickly.
* React + Vite gives a great developer experience.

---

## Scaffold

```text
gateorix/
├── cli/
│   ├── src/
│   │   ├── commands/
│   │   │   ├── init.ts
│   │   │   ├── dev.ts
│   │   │   ├── build.ts
│   │   │   ├── doctor.ts
│   │   │   ├── add-runtime.ts
│   │   │   └── add-plugin.ts
│   │   └── index.ts
│   └── package.json
├── host-core/
│   ├── src/
│   │   ├── app/
│   │   ├── window/
│   │   ├── ipc/
│   │   ├── permissions/
│   │   ├── plugins/
│   │   └── runtime/
│   └── README.md
├── sdk/
│   ├── js/
│   ├── python/
│   ├── dotnet/
│   ├── go/
│   └── swift/
├── templates/
│   ├── react-python/
│   │   ├── frontend/
│   │   ├── backend/
│   │   ├── gateorix.config.json
│   │   └── manifest.json
│   └── vanilla/
├── plugins/
│   ├── filesystem/
│   ├── process/
│   ├── notifications/
│   └── clipboard/
├── docs/
│   ├── architecture.md
│   ├── security.md
│   ├── adapter-protocol.md
│   └── roadmap.md
├── examples/
│   └── hello-react-python/
└── README.md
```

---

## Initial core contracts

### App manifest example

```json
{
  "name": "hello-gateorix",
  "version": "0.1.0",
  "frontend": {
    "devUrl": "http://localhost:5173",
    "entry": "dist/index.html"
  },
  "runtime": {
    "type": "python",
    "entry": "backend/main.py"
  },
  "permissions": {
    "filesystem": ["./data"],
    "process": false,
    "notifications": true,
    "clipboard": true
  },
  "windows": [
    {
      "id": "main",
      "title": "Gateorix App",
      "width": 1200,
      "height": 800
    }
  ]
}
```

### IPC request example

```json
{
  "id": "req-1001",
  "channel": "filesystem.readText",
  "payload": {
    "path": "./data/notes.txt"
  }
}
```

### IPC response example

```json
{
  "id": "req-1001",
  "ok": true,
  "payload": {
    "content": "Hello from Gateorix"
  }
}
```

---

## Sample CLI experience

```bash
gateorix init my-app --template react-python
gateorix dev
gateorix build
gateorix doctor
gateorix add runtime python
gateorix add plugin filesystem
```

---

## Phase plan

### Phase 1 — Foundation

* Finalize architecture
* Pick host core language
* Implement CLI skeleton
* Create manifest format
* Build React + Python starter template

### Phase 2 — Bridge and runtime

* Implement secure IPC
* Add Python runtime adapter
* Add basic filesystem and notification plugins
* Add development mode with hot reload

### Phase 3 — Desktop features

* Window API
* Menus, dialogs, tray
* Clipboard and shell plugins
* Logging and diagnostics

### Phase 4 — Packaging

* Native app packaging for Windows, macOS, Linux
* Runtime bundling strategy
* Signing and release workflow

### Phase 5 — Expansion

* Go, C#, F#, Swift adapters
* Plugin marketplace concept
* Binary IPC optimization
* Sandboxing and advanced security

---

## Suggested roadmap priorities

1. Prove the concept with React + Python.
2. Keep the bridge secure and minimal.
3. Make the CLI feel excellent from day one.
4. Add more runtimes only after the adapter protocol is stable.

---

![alt text](logo.png)
