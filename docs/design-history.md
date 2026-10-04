# Historical Design Notes

This is a condensed record of early proposals, not implementation instructions
or evidence that a feature is complete. Current behavior belongs in the
[architecture](architecture.md), [security model](security.md),
[adapter protocol](adapter-protocol.md), and [CLI guide](../cli/README.md).
Future work belongs in the [roadmap](roadmap.md).

## Provenance

The following documents were consolidated during the October 4, 2026 cleanup.
Their original text remains in Git history at commit `9819627`.

| Former document | Preserved subject |
|---|---|
| `Plan.md` | Thin native shell and language sidecars |
| `Structure.md` | Product direction, layers, initial contracts and phase plan |
| `docs/claudeprompt.md` | HTTP, native IPC and generator sequence |
| `docs/claudeprompt2.md` | Expanded sequence with persistent app shell |
| `docs/claudeprompt3.md` | Shared acceptance criteria for backend examples |
| `docs/NPMcommands.md` | CLI command and scaffolding requirements |
| `docs/perplexityprompt.md` | Early integration sketches and research references |

For example, `git show 9819627:docs/claudeprompt3.md` retrieves an original.
Repeated prompts, speculative size estimates, and obsolete command snippets
were removed from the active documentation rather than maintained as competing
guides. Historical phase numbers do not map exactly to the current roadmap.

## Product and Architecture Intent

Gateorix combines a web frontend with a thin native host, language adapters,
permission-checked plugins, and OS packaging. The selected tagline is
"Web UI. Native power." Earlier alternatives emphasized a gateway between web
and system, web-speed development, and native reach.

The early design considered Rust or C++ for the host and TypeScript or Rust
for the CLI, with React and Python as the first vertical slice. The repository
now uses a Rust host and TypeScript CLI. The original layering remains useful:
frontend, bridge, host, runtime adapters, plugins, and packaging.

The sidecar proposal keeps window management and OS integration in the native
shell while business logic runs in a separate process. Go binaries, .NET
self-contained or Native AOT builds, and C++ binaries were considered for
distribution. These were packaging options, not verified installer guarantees.
Objective-C was an early adapter idea, not a claim of current support.

Initial contracts specified request IDs, channel namespaces, JSON payloads,
correlated responses, frontend/runtime configuration, window definitions, and
explicit filesystem/process/notification/clipboard grants. Use the current
protocol and configuration schema instead of the old sample manifests.

## Incremental Implementation Sequence

1. Prove an HTTP frontend/backend round trip with loading and error states.
2. Wrap the frontend in Tauri and repeat the round trip through native IPC.
3. Build an app shell before generating further projects: navbar, theme,
   profile, login and logout.
4. Add a generator that asks for a backend language, UI framework and output
   folder, then copies the shell and IPC integration, installs dependencies,
   opens the project and launches development.

The app-shell acceptance criteria were theme persistence across restarts,
editable profile fields saved to disk, successful and failed demo login states,
and a logout that resets the UI. Hardcoded credentials were intended only for
the demo, not as a production authentication design.

The proposed VS Code generator included recommended editor settings, framework
templates, IPC helpers, window configuration, and a packaged extension smoke
test. Early names such as `tauri-gen` and `tauri-scaffolder`, and the suggested
Node backend option, were exploratory rather than current public contracts.

## Examples and CLI Requirements

The original React example plan covered Go, C#, F#, and C++ with the same shell
and acceptance checklist: launch, login, toggle theme, save profile, greet,
restart and verify persistence. The C++ proposal included CMake or Cargo build
integration and cross-platform compilation notes. Each example was to document
its own prerequisites, installation, execution and IPC behavior.

The CLI proposal covered `init`, `dev`, `build`, `doctor`, `add runtime`, and
`add plugin`. It requested UI/backend selection, placeholder replacement,
dependency installation, clear diagnostics, runtime scaffolding, and plugin
dependency/registration updates. Filesystem, process, notifications and clipboard
were the initial plugin targets. The former `packages/gateorix-cli` layout and
unscoped npm package name were superseded by `cli` and `@gateorixjs/cli`.

Early requests for automatic doctor checks, per-step `PROGRESS.md` files, and
inline glue comments were planning preferences, not established repository
requirements. The recurring intent was to test each increment and obtain review
before moving to the next. See the CLI guide for actual command behavior.

## Longer-Term Ideas

The proposals progressed from a React/Python proof of concept through secure
IPC, desktop APIs, runtime bundling, signing, more adapters, a plugin marketplace,
binary transport, sandboxing and cloud-connected plugins. These ideas are not
additional commitments; the roadmap owns prioritization and status.

## Original References and Artwork

These links preserve the research trail, not an endorsement of the old snippets
or their compatibility with current dependencies:

- [create-tauri-app](https://github.com/tauri-apps/create-tauri-app)
- [Historical windows and webviews documentation](https://jonaskruckenberg.github.io/tauri-docs-wip/development/windows-and-webviews.html)
- [Tauri v2](https://v2.tauri.app)
- [Tauri philosophy](https://v2.tauri.app/about/philosophy/)
- [Tauri architecture reference](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md)

The original [sidecar illustration](assets/image-1.png) and
[logo concept](assets/logo.png) are retained. An additional
[unclassified image](assets/image.png) had no tracked references before cleanup;
it is preserved pending owner review rather than deleted based on that alone.