#!/usr/bin/env node
/**
 * Sync repo-root `examples/` into `cli/templates/` so templates ship
 * with the published npm package.
 *
 * Runs automatically via `npm run prepack` before publish.
 */
import { cpSync, existsSync, mkdirSync, readdirSync, rmSync } from "node:fs";
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

console.log(`[sync-templates] copied ${count} templates -> cli/templates/`);
