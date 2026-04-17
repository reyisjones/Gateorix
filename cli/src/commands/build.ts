/**
 * `gateorix build` — build the app for production.
 */

interface BuildOptions {
  release: boolean;
}

export async function buildCommand(options: BuildOptions): Promise<void> {
  const mode = options.release ? "release" : "debug";
  console.log(`\n  Building Gateorix app (${mode})...`);

  // TODO: Phase 3 implementation
  // 1. Build frontend assets (vite build)
  // 2. Compile host core (cargo build --release)
  // 3. Bundle runtime adapter sidecar
  // 4. Package into OS-native installer

  console.log("  [build command not yet implemented — coming in Phase 3]");
}
