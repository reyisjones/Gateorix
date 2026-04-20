/**
 * `gateorix build` — build the app for production.
 *
 * Steps:
 *   1. Build frontend assets (Vite)
 *   2. Build Tauri app (if src-tauri exists) — produces native installer
 *   3. Report output location
 */

import { execSync } from "node:child_process";
import path from "node:path";
import fs from "fs-extra";
import chalk from "chalk";

interface BuildOptions {
  release: boolean;
}

function run(cmd: string, cwd: string, label: string): void {
  console.log(`  ${chalk.cyan("→")} ${label}`);
  try {
    execSync(cmd, { cwd, stdio: "inherit" });
  } catch {
    console.error(chalk.red(`\n  Build failed at step: ${label}`));
    process.exit(1);
  }
}

export async function buildCommand(options: BuildOptions): Promise<void> {
  const projectRoot = process.cwd();
  const configPath = path.join(projectRoot, "gateorix.config.json");

  if (!(await fs.pathExists(configPath))) {
    console.error(chalk.red("\n  No gateorix.config.json found in the current directory."));
    console.error(chalk.dim("  Run this command from your Gateorix project root.\n"));
    process.exit(1);
  }

  const config = await fs.readJson(configPath);
  const mode = options.release ? "release" : "debug";
  console.log(chalk.bold(`\n  Building ${config.name || "Gateorix app"} (${mode})\n`));

  const frontendDir = path.join(projectRoot, "frontend");
  const hasTauri = await fs.pathExists(path.join(frontendDir, "src-tauri"));

  // 1. Install frontend dependencies if needed
  const nodeModules = path.join(frontendDir, "node_modules");
  if (await fs.pathExists(path.join(frontendDir, "package.json")) && !(await fs.pathExists(nodeModules))) {
    run("npm install", frontendDir, "Installing frontend dependencies");
  }

  // 2. Build frontend
  if (await fs.pathExists(path.join(frontendDir, "package.json"))) {
    run("npm run build", frontendDir, "Building frontend assets");
  }

  // 3. Build Tauri app (native bundle) or just report frontend build
  if (hasTauri) {
    const tauriArgs = options.release ? "npx tauri build" : "npx tauri build --debug";
    run(tauriArgs, frontendDir, `Building Tauri app (${mode})`);

    // Report output
    const bundleDir = path.join(frontendDir, "src-tauri", "target", mode === "release" ? "release" : "debug", "bundle");
    console.log(chalk.bold("\n  Build complete!"));
    if (await fs.pathExists(bundleDir)) {
      console.log(`  ${chalk.green("→")} Output: ${path.relative(projectRoot, bundleDir)}`);
      try {
        const dirs = await fs.readdir(bundleDir);
        for (const d of dirs) {
          const stat = await fs.stat(path.join(bundleDir, d));
          if (stat.isDirectory()) {
            console.log(`    ${chalk.dim("•")} ${d}/`);
          }
        }
      } catch {
        // bundle dir listing is informational only
      }
    }
  } else {
    console.log(chalk.bold("\n  Frontend build complete!"));
    const distDir = path.join(frontendDir, "dist");
    if (await fs.pathExists(distDir)) {
      console.log(`  ${chalk.green("→")} Output: ${path.relative(projectRoot, distDir)}`);
    }
    console.log(chalk.yellow("\n  Note: No src-tauri found. To produce a native installer,"));
    console.log(chalk.yellow("  run `npx tauri init` in the frontend directory first."));
  }

  console.log("");
}
