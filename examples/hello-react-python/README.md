# Hello React + Python

A full-featured Gateorix example application with a React frontend, Python backend, and Tauri desktop shell.

## Features

- **App shell** — Navbar with logo, navigation, and user controls
- **Dark / Light theme** — Toggle persists to disk via Tauri settings
- **Profile page** — Edit display name and email, saved to `settings.json`
- **Login / Logout** — Hardcoded demo credentials with Rust-side validation
- **Greet demo** — Frontend → Rust → Python sidecar IPC round trip
- **Dual IPC modes** — Tauri native (production) and HTTP bridge (browser dev)

## Demo Credentials

| Username | Password | Display Name |
|----------|----------|-------------|
| `admin`  | `gateorix` | Admin     |
| `demo`   | `demo`     | Demo User |

## Structure

```
hello-react-python/
├── frontend/
│   ├── src/
│   │   ├── main.tsx         # React app shell (Navbar, Home, Profile, Login pages)
│   │   └── styles.css       # Dark/light theme CSS with custom properties
│   ├── src-tauri/
│   │   ├── src/
│   │   │   ├── lib.rs       # Tauri commands: invoke_backend, settings, login/logout
│   │   │   └── main.rs      # Entry point
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   ├── index.html
│   └── package.json
├── backend/
│   └── main.py              # Python sidecar (stdio) / dev bridge (--http on :3001)
└── gateorix.config.json
```

## Running

### Desktop (Tauri)

```bash
cd frontend
npm install
npx tauri dev
```

This starts Vite on port 5173, compiles the Rust backend, spawns the Python sidecar, and opens a native window.

### Browser Development (no Tauri)

```bash
# Terminal 1 — Python HTTP bridge
cd backend && python main.py --http

# Terminal 2 — Vite dev server
cd frontend && npm install && npm run dev
```

Open `http://localhost:5173` in your browser. IPC falls back to the HTTP bridge on port 3001. Login and settings features require Tauri mode.

### Test Python Backend Directly

```bash
echo '{"id":"1","channel":"runtime.greet","payload":{"name":"Alice"}}' | python backend/main.py
```

## Architecture

```
React (main.tsx)
  ├─ isTauri? ──► @tauri-apps/api/core invoke()
  │                 └─► lib.rs Tauri commands
  │                       ├─ invoke_backend → Python sidecar (stdio)
  │                       ├─ get_settings / save_settings → settings.json
  │                       └─ login / logout → hardcoded credential check
  └─ !isTauri ──► fetch() HTTP bridge (port 3001)
                    └─► main.py (--http mode)
```

Settings are stored at `%APPDATA%/com.gateorix.hello/settings.json` (Windows) or the platform equivalent.
