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
6. ✅ Approve before moving to Phase 2.5

---

**Phase 2.5 — Tauri Happy Path UI (shell before the extension)**

Before building the generator, prove out Tauri's frontend + backend capabilities with a minimal but real app shell. Keep it simple — just enough to demo the full stack working together.

*Build these features one at a time, test and approve each:*

1. **App shell layout** — top navbar with logo, theme toggle, profile button, login button. Clean, minimal. No framework complexity yet.

2. **Dark / Light theme toggle**
   - Button in navbar switches between dark and light mode
   - Theme stored via Tauri's backend (`tauri-plugin-store` or a simple `invoke()` call that writes to a local file)
   - Frontend reads theme on launch and applies it — persists across restarts
   - ✅ Test: toggle survives app close and reopen

3. **Profile page**
   - Profile button opens a simple profile view (same window or side panel)
   - Shows: avatar placeholder, display name, email field (editable)
   - Save button calls a Tauri command to persist data to local storage or a JSON file on disk
   - ✅ Test: edit name, save, close app, reopen — data persists

4. **Login page**
   - Login button navigates to a simple login form: username + password fields + submit
   - On submit: frontend calls `invoke('login', { username, password })` → Rust/backend validates (hardcoded happy path credentials for now)
   - On success: shows a "Welcome, {name}" state in the navbar, login button becomes logout
   - On failure: shows inline error message
   - ✅ Test: happy path login works, state reflects in navbar

5. **Logout**
   - Logout clears session state via a backend `invoke('logout')` command
   - Navbar returns to logged-out state
   - ✅ Test: logout resets UI, profile button disabled or hidden

6. ✅ Full Phase 2.5 review — all features working together before continuing

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
   - App shell from Phase 2.5 baked in: theme toggle, profile page, login page
   - `.vscode/settings.json` with recommended extensions
5. **Install dependencies** — extension runs `npm install` and backend package install in terminal
6. **Open and launch** — opens the new project folder, runs `npm run tauri dev`
7. ✅ Test full generation flow, approve output before publishing

---

**Reusable Prompt Template** *(use this pattern for any similar process)*

```
Goal: [what we're building]
Approach: gradual — one step at a time, test before continuing
Phase 1: [foundation layer]      → test → approve
Phase 2: [integration layer]     → test → approve
Phase 2.5: [happy path UI demo]  → test → approve
Phase 3: [tooling/automation]    → test → approve
Rule: never combine steps. show output after each. wait for explicit go-ahead.
```

---

**Ground rules for all phases**

- One step per response — show result, wait for approval
- Document each completed step in a running `PROGRESS.md`
- If a step fails, diagnose before moving forward — never skip
- The Phase 2.5 app shell is the canonical template — Phase 3 generator includes it by default
- All generated code includes inline comments explaining the Tauri glue