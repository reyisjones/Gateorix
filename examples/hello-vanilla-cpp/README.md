# Hello Gateorix — React + C++

A complete desktop application example using a **React** frontend with a **C++17** backend, powered by the Gateorix framework and Tauri v2.

## What It Does

- **Greet Demo** — Type a name and receive a greeting from the C++ backend via IPC
- **Dark/Light Theme** — Toggle with settings persisted to disk
- **Profile Page** — Edit and save profile data (Tauri native mode)
- **Login Page** — Demo authentication (`admin / gateorix` or `demo / demo`)
- **Native Menus & Tray** — File menu, View menu, system tray icon

## Architecture

```
┌──────────────┐   JSON/stdio    ┌──────────────────┐
│  Tauri Shell │ ◄────────────► │  C++ Backend      │
│  (Rust/WRY)  │   (sidecar)    │  (compiled binary) │
└──────┬───────┘                 └──────────────────┘
       │ IPC
┌──────┴───────┐
│  React App   │
│  (Vite + TS) │
└──────────────┘
```

**Production**: Tauri spawns the pre-compiled C++ binary as a sidecar process; communication via JSON over stdin/stdout.  
**Development**: The C++ binary runs an HTTP bridge on `localhost:3001`; React dev server on `localhost:5173`.

> Unlike the Python/Go/.NET examples, C++ requires a separate **compile step** before running.

## Prerequisites

| Tool      | Version | Install                                          |
|-----------|---------|--------------------------------------------------|
| Node.js   | ≥ 18    | https://nodejs.org                               |
| CMake     | ≥ 3.16  | https://cmake.org/download                       |
| C++ 17    | —       | MSVC / GCC / Clang (see below)                   |
| Rust      | ≥ 1.77  | https://rustup.rs                                |
| Tauri CLI | v2      | `npm install -g @tauri-apps/cli`                 |

### C++ Compiler Setup

**Windows** (recommended):
- Install [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Select "Desktop development with C++" workload
- CMake will use MSVC automatically

**macOS**:
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian)**:
```bash
sudo apt install build-essential cmake
```

## Quick Start (Dev Mode)

### 1. Build the C++ backend

```bash
cd backend
mkdir build && cd build
cmake ..
cmake --build .
```

**Windows (MSVC)**: The binary will be at `build/Debug/hello_gateorix_cpp.exe`  
**Linux/macOS**: The binary will be at `build/hello_gateorix_cpp`

### 2. Run the HTTP dev bridge

```bash
# Windows
.\build\Debug\hello_gateorix_cpp.exe --http

# Linux/macOS
./build/hello_gateorix_cpp --http
```

You should see:
```
  Gateorix HTTP dev bridge (C++)
  → listening on http://localhost:3001/invoke
  → CORS allowed: http://localhost:5173
```

### 3. Start the React frontend

```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173` in your browser. Type a name and click **Greet** — the greeting comes from C++.

### 4. Run as a native desktop app (optional)

First build the backend (step 1), then:

```bash
cd frontend
npm run tauri dev
```

The Tauri shell will locate the compiled binary and spawn it as a sidecar.

## How IPC Works

### Stdio (Production)

The Tauri Rust shell spawns the C++ binary as a child process. Every command is a JSON line written to stdin:

```json
{"id":"abc-123","channel":"runtime.greet","payload":{"name":"Alice"}}
```

The C++ backend reads the line, dispatches to the handler, and writes a JSON response to stdout:

```json
{"id":"abc-123","ok":true,"payload":{"message":"Hello from C++, Alice! Welcome to Gateorix."}}
```

### HTTP (Development)

With `--http`, the C++ binary starts a minimal single-threaded HTTP server using raw sockets (no external dependencies). The React app posts to `http://localhost:3001/invoke` with the same JSON body. CORS headers allow requests from the Vite dev server at `localhost:5173`.

### JSON Parsing

The C++ backend includes a **minimal JSON parser** (string/object extraction) with zero external dependencies. For production use, consider integrating [nlohmann/json](https://github.com/nlohmann/json) or [RapidJSON](https://rapidjson.org/).

## Cross-Platform Notes

| Platform | Compiler | Socket Library | Binary Name               |
|----------|----------|----------------|---------------------------|
| Windows  | MSVC     | Winsock2       | `hello_gateorix_cpp.exe`  |
| macOS    | Clang    | POSIX          | `hello_gateorix_cpp`      |
| Linux    | GCC      | POSIX          | `hello_gateorix_cpp`      |

The `CMakeLists.txt` automatically links `ws2_32` on Windows for socket support. The Tauri `lib.rs` checks multiple build output directories (`build/Debug/`, `build/Release/`, `build/`) to find the binary.

## Project Structure

```
hello-vanilla-cpp/
├── gateorix.config.json      # Gateorix project manifest
├── backend/
│   ├── main.cpp               # C++ backend (greet + echo + HTTP server)
│   └── CMakeLists.txt         # CMake build configuration
└── frontend/
    ├── index.html
    ├── package.json
    ├── src/
    │   ├── main.tsx           # React app (Navbar, Home, Profile, Login)
    │   └── styles.css         # Dark/light theme CSS
    └── src-tauri/
        ├── Cargo.toml         # Rust dependencies
        ├── tauri.conf.json    # Tauri configuration
        ├── capabilities/      # Permission declarations
        └── src/
            └── lib.rs         # Tauri commands + C++ binary sidecar spawn
```

## Adding a New Backend Command

1. Add a handler function in `backend/main.cpp`:
   ```cpp
   static std::string handle_mycommand(const std::string &payload) {
       return "{\"result\":\"done\"}";
   }
   ```

2. Register it in the `handlers` map:
   ```cpp
   {"mycommand", handle_mycommand},
   ```

3. Call it from the React frontend:
   ```typescript
   const data = await invokeBackend("runtime.mycommand", { key: "value" });
   ```

## Release Build

For a smaller, optimized binary:

```bash
cd backend/build
cmake .. -DCMAKE_BUILD_TYPE=Release
cmake --build . --config Release
```

## License

MIT — see [LICENSE](../../LICENSE) in the repository root.
