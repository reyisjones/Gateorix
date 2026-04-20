**Implement `gateorix` CLI — NPM Package with Full Command Suite**

---

Build the `gateorix` CLI as a proper NPM package. One command at a time, test and approve each before moving to the next.

---

**Package setup first**

Before any commands, scaffold the package itself:

1. Create `packages/gateorix-cli/` with `package.json` — name: `gateorix`, bin: `gateorix`
2. Entry point: `src/index.ts` — wire CLI framework (use `commander` or `citty`)
3. Add `tsconfig.json`, build script outputs to `dist/`
4. Test: `npm install -g .` → `gateorix --help` shows command list
5. ✅ Approve before adding any commands

---

**Command 1 — `gateorix init <name>`**

Scaffold a new project from a template.

1. Prompt user with QuickPick-style interactive questions:
   - Backend language: Rust · Go · Python · C# · F# · C++
   - UI framework: React · Vue · Svelte · Solid · Vanilla
2. Copy the matching `examples/hello-react-{lang}` template into `./<name>/`
3. Replace all placeholder names with `<name>` throughout files
4. Run `npm install` inside the new project folder
5. Print next steps: `cd <name> && gateorix dev`
6. ✅ Test: `gateorix init my-app` → full project folder generated and ready

---

**Command 2 — `gateorix dev`**

Start the app in development mode.

1. Detect project root (find `tauri.conf.json` walking up from cwd)
2. Run `npm run tauri dev` with live output piped to terminal
3. Watch for common errors (missing Rust, wrong Node version) and print friendly fix hints
4. ✅ Test: run inside a generated project → app launches with hot reload

---

**Command 3 — `gateorix build`**

Build the app for production.

1. Run pre-build checks: dependencies installed, backend compiles cleanly
2. Run `npm run tauri build` with progress output
3. On success: print output path of the generated binary/installer
4. On failure: print the failing step and a suggested fix
5. ✅ Test: run inside a generated project → installer produced in `src-tauri/target/release/bundle/`

---

**Command 4 — `gateorix doctor`**

Check environment and dependencies.

Verify and report status for each requirement:

| Check | Tool | Pass / Fail |
|---|---|---|
| Node.js | ≥ 18 | version detected |
| Rust + Cargo | stable | version detected |
| Tauri CLI | `@tauri-apps/cli` | version detected |
| WebView2 (Windows) | registry check | present / missing |
| Xcode CLT (macOS) | `xcode-select` | present / missing |
| Active backend | from `gateorix.config.json` | installed / missing |

Print a color-coded summary. For each failure, print the exact install command to fix it.

✅ Test: run on a clean machine → all failures shown with fix instructions

---

**Command 5 — `gateorix add runtime <lang>`**

Add a runtime adapter to an existing project.

Supported values: `python` · `go` · `dotnet` · `swift`

1. Read `gateorix.config.json` in project root — check no runtime already set
2. Copy the matching `src-{lang}/` sidecar scaffold into the project
3. Update `src-tauri/tauri.conf.json` to register the sidecar binary
4. Update `gateorix.config.json` with `"runtime": "<lang>"`
5. Print: prerequisites for that runtime + how to run `gateorix dev`
6. ✅ Test: `gateorix add runtime python` inside a vanilla project → Python sidecar wired and working

---

**Command 6 — `gateorix add plugin <name>`**

Add a plugin to an existing project.

Supported values: `filesystem` · `process` · `notifications` · `clipboard`

Each plugin does three things:
1. Installs the matching `@tauri-apps/plugin-*` NPM package
2. Adds the Rust crate to `src-tauri/Cargo.toml`
3. Registers the plugin in `src-tauri/main.rs` and adds a typed `invoke()` helper to `src/lib/plugins/`

Plugin details:

| Plugin | NPM package | Rust crate |
|---|---|---|
| `filesystem` | `@tauri-apps/plugin-fs` | `tauri-plugin-fs` |
| `process` | `@tauri-apps/plugin-process` | `tauri-plugin-process` |
| `notifications` | `@tauri-apps/plugin-notification` | `tauri-plugin-notification` |
| `clipboard` | `@tauri-apps/plugin-clipboard-manager` | `tauri-plugin-clipboard-manager` |

✅ Test: `gateorix add plugin clipboard` → clipboard read/write working in the app via `invoke()`

---

**`gateorix.config.json` — project config file**

Created by `init`, updated by `add runtime` and `add plugin`:

```json
{
  "name": "my-app",
  "runtime": "go",
  "ui": "react",
  "plugins": ["clipboard", "filesystem"],
  "version": "0.1.0"
}
```

---

**Package folder layout**

```
packages/gateorix-cli/
├── src/
│   ├── index.ts           ← CLI entry, command registration
│   ├── commands/
│   │   ├── init.ts
│   │   ├── dev.ts
│   │   ├── build.ts
│   │   ├── doctor.ts
│   │   ├── add-runtime.ts
│   │   └── add-plugin.ts
│   ├── templates/         ← symlinks or copies of examples/hello-react-{lang}
│   └── utils/
│       ├── detect.ts      ← find project root, read config
│       ├── exec.ts        ← run shell commands with output
│       └── logger.ts      ← color-coded output helpers
├── package.json
├── tsconfig.json
└── README.md
```

---

**Ground rules**

- One command implemented and tested at a time — approve before the next
- Every command prints a clear success or failure message — no silent exits
- `doctor` runs automatically at the start of `init`, `dev`, and `build` — warns but does not block
- All commands update `PROGRESS.md` with a timestamped entry on completion