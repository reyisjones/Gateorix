/**
 * `gateorix add plugin <name>` — add a plugin to the project.
 *
 * For each plugin:
 *   1. Install the matching @tauri-apps/plugin-* NPM package
 *   2. Add the Rust crate to src-tauri/Cargo.toml
 *   3. Register the plugin in src-tauri/src/lib.rs (or main.rs)
 *   4. Update gateorix.config.json
 */

import path from "node:path";
import { execSync } from "node:child_process";
import fs from "fs-extra";
import chalk from "chalk";

interface PluginInfo {
  npm: string;
  crate: string;
  registerCall: string;
}

const PLUGIN_MAP: Record<string, PluginInfo> = {
  filesystem: {
    npm: "@tauri-apps/plugin-fs",
    crate: "tauri-plugin-fs",
    registerCall: ".plugin(tauri_plugin_fs::init())",
  },
  process: {
    npm: "@tauri-apps/plugin-process",
    crate: "tauri-plugin-process",
    registerCall: ".plugin(tauri_plugin_process::init())",
  },
  notifications: {
    npm: "@tauri-apps/plugin-notification",
    crate: "tauri-plugin-notification",
    registerCall: ".plugin(tauri_plugin_notification::init())",
  },
  clipboard: {
    npm: "@tauri-apps/plugin-clipboard-manager",
    crate: "tauri-plugin-clipboard-manager",
    registerCall: ".plugin(tauri_plugin_clipboard_manager::init())",
  },
};

const AVAILABLE_PLUGINS = Object.keys(PLUGIN_MAP);

export async function addPluginCommand(name: string): Promise<void> {
  if (!AVAILABLE_PLUGINS.includes(name)) {
    console.error(chalk.red(`\n  Error: unknown plugin "${name}".`));
    console.error(`  Available: ${AVAILABLE_PLUGINS.join(", ")}\n`);
    process.exit(1);
  }

  const projectRoot = process.cwd();
  const configPath = path.join(projectRoot, "gateorix.config.json");

  if (!(await fs.pathExists(configPath))) {
    console.error(chalk.red("\n  No gateorix.config.json found in the current directory."));
    console.error(chalk.dim("  Run this command from your Gateorix project root.\n"));
    process.exit(1);
  }

  const plugin = PLUGIN_MAP[name];
  console.log(chalk.bold(`\n  Adding ${name} plugin...\n`));

  // 1. Install NPM package in frontend/
  const frontendDir = path.join(projectRoot, "frontend");
  if (await fs.pathExists(path.join(frontendDir, "package.json"))) {
    console.log(`  ${chalk.cyan("→")} Installing ${plugin.npm}...`);
    try {
      execSync(`npm install ${plugin.npm}`, { cwd: frontendDir, stdio: "pipe" });
      console.log(`  ${chalk.green("✓")} NPM package installed`);
    } catch {
      console.log(chalk.yellow(`  ! Failed to install ${plugin.npm} — install manually`));
    }
  }

  // 2. Add Rust crate to src-tauri/Cargo.toml
  const tauriDir = path.join(frontendDir, "src-tauri");
  const cargoPath = path.join(tauriDir, "Cargo.toml");
  if (await fs.pathExists(cargoPath)) {
    let cargo = await fs.readFile(cargoPath, "utf-8");
    if (!cargo.includes(plugin.crate)) {
      // Add under [dependencies]
      const depMarker = "[dependencies]";
      const idx = cargo.indexOf(depMarker);
      if (idx !== -1) {
        const insertPos = idx + depMarker.length;
        cargo = cargo.slice(0, insertPos) + `\n${plugin.crate} = "2"` + cargo.slice(insertPos);
        await fs.writeFile(cargoPath, cargo, "utf-8");
        console.log(`  ${chalk.green("✓")} Added ${plugin.crate} to Cargo.toml`);
      }
    } else {
      console.log(chalk.dim(`  • ${plugin.crate} already in Cargo.toml`));
    }
  }

  // 3. Register plugin in src-tauri/src/lib.rs (or main.rs)
  const libPath = path.join(tauriDir, "src", "lib.rs");
  const mainRsPath = path.join(tauriDir, "src", "main.rs");
  const rsPath = (await fs.pathExists(libPath)) ? libPath : mainRsPath;

  if (await fs.pathExists(rsPath)) {
    let rs = await fs.readFile(rsPath, "utf-8");
    if (!rs.includes(plugin.crate.replace(/-/g, "_"))) {
      // Insert plugin registration before .run() or .invoke_handler()
      const runPattern = /\.run\(tauri::generate_context!\(\)\)/;
      const match = rs.match(runPattern);
      if (match && match.index !== undefined) {
        const insertAt = match.index;
        rs = rs.slice(0, insertAt) + `${plugin.registerCall}\n        ` + rs.slice(insertAt);
        await fs.writeFile(rsPath, rs, "utf-8");
        console.log(`  ${chalk.green("✓")} Registered plugin in ${path.basename(rsPath)}`);
      } else {
        console.log(chalk.yellow(`  ! Could not auto-register — add ${plugin.registerCall} manually`));
      }
    } else {
      console.log(chalk.dim(`  • Plugin already registered in ${path.basename(rsPath)}`));
    }
  }

  // 4. Update gateorix.config.json
  const config = await fs.readJson(configPath);
  if (!config.plugins) config.plugins = [];
  if (!config.plugins.includes(name)) {
    config.plugins.push(name);
    await fs.writeJson(configPath, config, { spaces: 2 });
    console.log(`  ${chalk.green("✓")} Updated gateorix.config.json`);
  }

  console.log(chalk.green(`\n  ✓ Plugin "${name}" added successfully\n`));
}
