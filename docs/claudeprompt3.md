**Create Sample Projects — Hello React + Backend Examples**

---

Build four minimal but complete Tauri sample projects. Each follows the same happy path shell from Phase 2.5 — same UI, different backend language. One at a time, test and approve before the next.

---

**Shared structure across all four projects**

Every example uses the same frontend and app shell:
- React + Vite (TypeScript)
- Navbar: theme toggle · profile button · login button
- Dark/light mode persisted via backend
- Profile page with save (persists to local file)
- Login page with hardcoded happy path credentials
- One demo `invoke()` call that returns a greeting from the backend — proves IPC works

The only thing that changes per project is `src-tauri/` and the backend language.

---

**Project 1 — `examples/hello-react-go`**
Backend: Go via `tauri-plugin-shell` sidecar or direct Go HTTP sidecar bridged to Tauri IPC
1. Scaffold Tauri + React frontend
2. Add Go sidecar: `src-go/main.go` with a `greet(name)` handler
3. Wire `invoke('greet')` → Go → returns `"Hello from Go, {name}!"`
4. Integrate app shell (theme, profile, login) — all state persisted by Go backend writing JSON to app data dir
5. ✅ Test: launch app, login, toggle theme, save profile, greet — all working

---

**Project 2 — `examples/hello-react-cs`**
Backend: C# via .NET sidecar
1. Scaffold Tauri + React frontend
2. Add C# sidecar: `src-dotnet/Program.cs` — minimal console app exposing IPC commands
3. Wire `invoke('greet')` → C# → returns `"Hello from C#, {name}!"`
4. Integrate app shell — state persisted by C# writing JSON to app data dir
5. ✅ Test: same happy path as Project 1

---

**Project 3 — `examples/hello-react-fs`**
Backend: F# via .NET sidecar
1. Scaffold Tauri + React frontend
2. Add F# sidecar: `src-fsharp/Program.fs` — functional style, same IPC surface as C#
3. Wire `invoke('greet')` → F# → returns `"Hello from F#, {name}!"`
4. Integrate app shell — state persisted by F# backend
5. ✅ Test: same happy path as Project 1

---

**Project 4 — `examples/hello-react-cpp`**
Backend: C++ via compiled sidecar binary
1. Scaffold Tauri + React frontend
2. Add C++ sidecar: `src-cpp/main.cpp` — compiled to binary, communicates via stdin/stdout JSON
3. Wire `invoke('greet')` → C++ binary → returns `"Hello from C++, {name}!"`
4. Add `build.rs` or CMake config to compile the binary as part of the Tauri build
5. Integrate app shell — state persisted by C++ writing JSON to app data dir
6. ✅ Test: same happy path as Project 1

---

**Shared folder layout per project**

```
examples/hello-react-{lang}/
├── src/                  ← React frontend (shared across all)
├── src-tauri/            ← Tauri config, commands, glue
├── src-{lang}/           ← Backend source (go / dotnet / fsharp / cpp)
├── tauri.conf.json
├── package.json
├── vite.config.ts
├── README.md             ← setup + run instructions for this backend
└── PROGRESS.md           ← completed steps log
```

---

**Ground rules**

- Build and test one project at a time — approve before the next
- Each README covers: prerequisites, install, run, how IPC works in that language
- All four projects must pass the same happy path checklist before any is considered done
- C++ project includes CMake setup and notes on cross-platform compilation