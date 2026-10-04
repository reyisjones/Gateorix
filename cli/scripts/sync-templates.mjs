#!/usr/bin/env node
/**
 * Sync repo-root `examples/` into `cli/templates/` so templates ship
 * with the published npm package.
 *
 * Runs automatically via `npm run prepack` before publish.
 */
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const cliRoot = resolve(__dirname, "..");
const repoRoot = resolve(cliRoot, "..");
const srcExamples = join(repoRoot, "examples");
const dstTemplates = join(cliRoot, "templates");

if (!existsSync(srcExamples)) {
  console.error(`[sync-templates] repo examples/ not found at ${srcExamples}`);
  process.exit(1);
}

if (existsSync(dstTemplates)) rmSync(dstTemplates, { recursive: true, force: true });
mkdirSync(dstTemplates, { recursive: true });

const skip = new Set(["node_modules", "target", "dist", ".git", "gen"]);
const entries = readdirSync(srcExamples, { withFileTypes: true });
let count = 0;

for (const entry of entries) {
  if (!entry.isDirectory()) continue;
  if (!entry.name.startsWith("hello-")) continue;
  const src = join(srcExamples, entry.name);
  const dst = join(dstTemplates, entry.name);
  cpSync(src, dst, {
    recursive: true,
    filter: (s) => {
      const base = s.split(/[\\/]/).pop() ?? "";
      if (skip.has(base)) return false;
      if (base === "Cargo.lock") return false;
      return true;
    },
  });
  count++;
}

for (const ui of ["react", "vue", "svelte", "solid", "vanilla"]) {
  const name = `hello-${ui}-rust`;
  const destination = join(dstTemplates, name);
  cpSync(join(dstTemplates, `hello-${ui}-go`), destination, { recursive: true });
  rmSync(join(destination, "backend"), { recursive: true, force: true });
  rmSync(join(destination, "frontend", "src-tauri"), { recursive: true, force: true });
  cpSync(join(repoRoot, "templates", "rust-backend"), join(destination, "backend"), {
    recursive: true,
    filter: (source) => !skip.has(source.split(/[\\/]/).pop()) && !source.endsWith("Cargo.lock"),
  });
  const sdkDestination = join(destination, "backend", "gateorix-adapter");
  mkdirSync(sdkDestination, { recursive: true });
  for (const entry of ["Cargo.toml", "src"]) {
    cpSync(join(repoRoot, "sdk", "rust", entry), join(sdkDestination, entry), { recursive: true });
  }
  cpSync(join(repoRoot, "LICENSE"), join(sdkDestination, "LICENSE"));
  const configPath = join(destination, "gateorix.config.json");
  const config = JSON.parse(readFileSync(configPath, "utf8"));
  config.name = name;
  config.runtime = { type: "rust", entry: "backend/Cargo.toml" };
  config.permissions = { filesystem: [], process: false, notifications: false, clipboard: false };
  writeFileSync(configPath, JSON.stringify(config, null, 2) + "\n");
  writeFileSync(join(destination, "README.md"), `# ${name}\n\nRust backend with ${ui} browser development. Run npm install in frontend, then gx dev from the project root.\n\nThe backend supports stdio and loopback HTTP development mode. Native Rust installer bundling is not configured.\n`);
  count++;
}

console.log(`[sync-templates] copied ${count} templates -> cli/templates/`);
