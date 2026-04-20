/**
 * Shell execution utilities — run commands with output handling.
 */

import { execSync, spawn, ChildProcess } from "node:child_process";
import { log } from "./logger.js";

export interface ExecResult {
  success: boolean;
  output: string;
  code: number | null;
}

/**
 * Run a shell command synchronously. Returns the result with exit info.
 */
export function exec(command: string, cwd?: string): ExecResult {
  try {
    const output = execSync(command, {
      cwd,
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    }).trim();
    return { success: true, output, code: 0 };
  } catch (err: unknown) {
    const e = err as { status?: number; stdout?: string; stderr?: string };
    return {
      success: false,
      output: (e.stderr || e.stdout || "").toString().trim(),
      code: e.status ?? 1,
    };
  }
}

/**
 * Run a shell command synchronously, piping output to the terminal.
 * Throws on non-zero exit.
 */
export function execPiped(command: string, cwd: string, label: string): void {
  log.step(label);
  try {
    execSync(command, { cwd, stdio: "inherit" });
  } catch {
    log.error(`Failed at step: ${label}`);
    process.exit(1);
  }
}

/**
 * Spawn a long-running process with labeled, color-coded output streaming.
 */
export function spawnLabeled(
  label: string,
  command: string,
  args: string[],
  cwd: string,
  colorFn: (s: string) => string,
  env?: Record<string, string>,
): ChildProcess {
  const child = spawn(command, args, {
    cwd,
    stdio: ["ignore", "pipe", "pipe"],
    shell: true,
    env: { ...process.env, ...env },
  });

  const prefix = colorFn(`[${label}]`);

  child.stdout?.on("data", (data: Buffer) => {
    for (const line of data.toString().split("\n").filter(Boolean)) {
      console.log(`  ${prefix} ${line}`);
    }
  });

  child.stderr?.on("data", (data: Buffer) => {
    for (const line of data.toString().split("\n").filter(Boolean)) {
      console.log(`  ${prefix} ${line}`);
    }
  });

  child.on("exit", (code) => {
    if (code !== null && code !== 0) {
      console.log(`  ${prefix} exited with code ${code}`);
    }
  });

  return child;
}
