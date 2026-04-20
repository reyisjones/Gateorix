/**
 * Project detection utilities — find project root, read config.
 */

import path from "node:path";
import fs from "fs-extra";

export interface GateorixConfig {
  name: string;
  version?: string;
  runtime?: {
    type: string;
    entry: string;
  };
  ui?: string;
  frontend?: {
    devUrl?: string;
    entry?: string;
  };
  permissions?: Record<string, unknown>;
  plugins?: string[];
  windows?: Array<{
    id: string;
    title: string;
    width: number;
    height: number;
  }>;
}

/**
 * Walk up from `startDir` looking for `gateorix.config.json`.
 * Returns the directory containing it, or null.
 */
export function findProjectRoot(startDir: string = process.cwd()): string | null {
  let dir = path.resolve(startDir);
  for (let i = 0; i < 20; i++) {
    if (fs.existsSync(path.join(dir, "gateorix.config.json"))) {
      return dir;
    }
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return null;
}

/**
 * Read and parse the project config. Returns null if not found.
 */
export async function readConfig(projectRoot?: string): Promise<GateorixConfig | null> {
  const root = projectRoot ?? findProjectRoot();
  if (!root) return null;
  const configPath = path.join(root, "gateorix.config.json");
  if (!(await fs.pathExists(configPath))) return null;
  try {
    return await fs.readJson(configPath) as GateorixConfig;
  } catch {
    return null;
  }
}

/**
 * Write the project config back to disk.
 */
export async function writeConfig(config: GateorixConfig, projectRoot?: string): Promise<void> {
  const root = projectRoot ?? findProjectRoot();
  if (!root) throw new Error("No project root found");
  await fs.writeJson(path.join(root, "gateorix.config.json"), config, { spaces: 2 });
}

/**
 * Check whether Tauri is set up in the frontend directory.
 */
export function hasTauri(projectRoot: string): boolean {
  return fs.existsSync(path.join(projectRoot, "frontend", "src-tauri"));
}

/**
 * Walk up from __dirname to find the package root (where examples/ lives).
 */
export function findPackageRoot(): string {
  let dir = path.resolve(__dirname, "..");
  for (let i = 0; i < 10; i++) {
    if (fs.existsSync(path.join(dir, "examples"))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return path.resolve(__dirname, "..");
}
