**Implementation Plan — HTTP Dev Bridge + Tauri Webview + VS Code Extension**

---

**Phase 1 — HTTP Dev Bridge (end-to-end flow first)**

Set up a local HTTP bridge so the frontend and backend can talk before any Tauri integration. One step at a time, test each before continuing.

1. Start a local HTTP server on the backend (e.g. `localhost:3000`)
2. Connect the frontend to hit that endpoint with a test request
3. Verify round-trip response renders correctly in the UI
4. Add error handling and loading states
5. ✅ Approve before moving to Phase 2

---

**Phase 2 — Tauri Webview Integration**

With the bridge working, wrap it in Tauri. Each step builds on the last.

1. Scaffold Tauri project around the existing frontend (`npm create tauri-app`)
2. Point Tauri's webview at the dev server URL during development
3. Wire one command: frontend calls `invoke()` → Tauri passes to backend → returns response
4. Replace the HTTP bridge call with the native Tauri command
5. Test the same end-to-end flow now running through Tauri IPC
6. ✅ Approve before moving to Phase 3

---

**Phase 3 — VS Code Extension: Tauri Project Generator**

A VS Code extension that scaffolds a full Tauri desktop app from a command palette prompt — backend language and UI framework of the user's choice, with all the Tauri glue pre-wired.

*Step-by-step build:*

1. **Scaffold the extension** — run `yo code`, choose TypeScript, name it `tauri-gen`
2. **Register a command** — `tauri-gen.newProject` shows in Command Palette
3. **Prompt the user** — QuickPick menus:
   - Backend language: Rust · Python · Go · Node
   - UI framework: React · Vue · Svelte · Solid · Vanilla
4. **Generate project structure** — extension writes files to a chosen workspace folder:
   - `src-tauri/` with `tauri.conf.json`, `Cargo.toml` or equivalent
   - Frontend scaffold matching chosen framework
   - Tauri glue: `invoke()` helper, IPC command stubs, window config
   - `.vscode/settings.json` with recommended extensions
5. **Install dependencies** — extension runs `npm install` and backend package install in terminal
6. **Open and launch** — opens the new project folder, runs `npm run tauri dev`
7. ✅ Test full generation flow, approve output before publishing

---

**Reusable Prompt Template** *(use this pattern for any similar process)*

```
Goal: [what we're building]
Approach: gradual — one step at a time, test before continuing
Phase 1: [foundation layer] → test → approve
Phase 2: [integration layer] → test → approve
Phase 3: [tooling/automation] → test → approve
Rule: never combine steps. show output after each. wait for explicit go-ahead.
```

---

**Ground rules for all phases**

- One step per response — show result, wait for approval
- Document each completed step in a running `PROGRESS.md`
- If a step fails, diagnose before moving forward — never skip
- All generated code includes inline comments explaining the Tauri glue