/**
 * `gateorix dev` — start the app in development mode.
 *
 * Orchestrates three processes:
 *   1. Frontend dev server (Vite)
 *   2. Backend sidecar (runtime adapter)
 *   3. Tauri host shell (if src-tauri exists)
 */

import { spawn, ChildProcess } from "node:child_process";
import path from "node:path";
import fs from "fs-extra";
import chalk from "chalk";

const children: ChildProcess[] = [];

function cleanup(): void {
  for (const child of children) {
    if (!child.killed) {
      child.kill();
    }
  }
}

function startProcess(
  label: string,
  command: string,
  args: string[],
  cwd: string,
  color: (s: string) => string,
  env?: Record<string, string>
): ChildProcess {
  const child = spawn(command, args, {
    cwd,
    stdio: ["ignore", "pipe", "pipe"],
    shell: true,
    env: { ...process.env, ...env },
  });

  const prefix = color(`[${label}]`);

  child.stdout?.on("data", (data: Buffer) => {
    const lines = data.toString().split("\n").filter(Boolean);
    for (const line of lines) {
      console.log(`  ${prefix} ${line}`);
    }
  });

  child.stderr?.on("data", (data: Buffer) => {
    const lines = data.toString().split("\n").filter(Boolean);
    for (const line of lines) {
      console.log(`  ${prefix} ${chalk.dim(line)}`);
    }
  });

  child.on("exit", (code) => {
    if (code !== null && code !== 0) {
      console.log(`  ${prefix} exited with code ${code}`);
    }
  });

  children.push(child);
  return child;
}

export async function devCommand(): Promise<void> {
  const projectRoot = process.cwd();
  const configPath = path.join(projectRoot, "gateorix.config.json");

  if (!(await fs.pathExists(configPath))) {
    console.error(chalk.red("\n  No gateorix.config.json found in the current directory."));
    console.error(chalk.dim("  Run this command from your Gateorix project root.\n"));
    process.exit(1);
  }

  const config = await fs.readJson(configPath);
  console.log(chalk.bold(`\n  Starting Gateorix dev mode — ${config.name || "project"}\n`));

  // Handle graceful shutdown
  process.on("SIGINT", () => {
    console.log(chalk.dim("\n  Shutting down..."));
    cleanup();
    process.exit(0);
  });
  process.on("SIGTERM", () => {
    cleanup();
    process.exit(0);
  });

  // 1. Start backend sidecar
  if (config.runtime?.entry) {
    const runtimeType = config.runtime.type || "python";
    const runtimeEntry = config.runtime.entry;
    const backendDir = path.join(projectRoot, path.dirname(runtimeEntry));
    const entryFile = path.basename(runtimeEntry);

    let cmd: string;
    let args: string[];

    if (runtimeType === "python") {
      cmd = process.platform === "win32" ? "python" : "python3";
      args = [entryFile, "--http"];
    } else if (runtimeType === "go") {
      cmd = "go";
      args = ["run", ".", "--http"];
    } else if (runtimeType === "dotnet" || runtimeType === "csharp" || runtimeType === "fsharp") {
      cmd = "dotnet";
      args = ["run"];
    } else if (runtimeType === "cpp") {
      // Look for the built binary in common CMake output dirs
      const buildDir = path.join(backendDir, "build");
      const binaryName = process.platform === "win32" ? "hello-gateorix-cpp.exe" : "hello-gateorix-cpp";
      const searchDirs = [
        path.join(buildDir, "Debug"),
        path.join(buildDir, "Release"),
        buildDir,
      ];
      let binaryPath = "";
      for (const d of searchDirs) {
        const candidate = path.join(d, binaryName);
        if (fs.existsSync(candidate)) {
          binaryPath = candidate;
          break;
        }
      }
      if (binaryPath) {
        cmd = binaryPath;
        args = [];
      } else {
        console.log(`  ${chalk.yellow("!")} C++ binary not found. Build with CMake first:`);
        console.log(chalk.dim(`    cd backend && cmake -B build && cmake --build build`));
        cmd = binaryName;
        args = [];
      }
    } else {
      cmd = runtimeType;
      args = [entryFile];
    }

    console.log(`  ${chalk.cyan("→")} Backend (${runtimeType}): ${runtimeEntry}`);
    startProcess("backend", cmd, args, backendDir, chalk.cyan);
  }

  // 2. Start frontend dev server
  const frontendDir = path.join(projectRoot, "frontend");
  const hasTauri = await fs.pathExists(path.join(frontendDir, "src-tauri"));

  if (hasTauri) {
    // Use Tauri dev — it starts both Vite and the Tauri shell
    console.log(`  ${chalk.magenta("→")} Frontend + Tauri: npx tauri dev`);
    startProcess("tauri", "npx", ["tauri", "dev"], frontendDir, chalk.magenta);
  } else if (await fs.pathExists(path.join(frontendDir, "package.json"))) {
    // Vite-only mode (browser dev)
    const devUrl = config.frontend?.devUrl || "http://localhost:5173";
    console.log(`  ${chalk.green("→")} Frontend: ${devUrl}`);
    startProcess("frontend", "npm", ["run", "dev"], frontendDir, chalk.green);
  } else {
    console.log(chalk.yellow("  ! No frontend/package.json found, skipping frontend server"));
  }

  console.log(chalk.dim("\n  Press Ctrl+C to stop all processes\n"));

  // Keep the process alive
  await new Promise<void>(() => {});
}
