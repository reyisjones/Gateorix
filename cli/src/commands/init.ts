/**
 * `gateorix init <name>` — scaffold a new Gateorix project.
 */

import path from "node:path";
import fs from "fs-extra";

interface InitOptions {
  template: string;
}

export async function initCommand(
  name: string,
  options: InitOptions
): Promise<void> {
  const targetDir = path.resolve(process.cwd(), name);

  if (await fs.pathExists(targetDir)) {
    console.error(`Error: directory "${name}" already exists.`);
    process.exit(1);
  }

  console.log(`\n  Creating Gateorix project: ${name}`);
  console.log(`  Template: ${options.template}\n`);

  // Create project directory structure
  await fs.ensureDir(targetDir);
  await fs.ensureDir(path.join(targetDir, "frontend"));
  await fs.ensureDir(path.join(targetDir, "backend"));

  // Write manifest
  const manifest = {
    name,
    version: "0.1.0",
    frontend: {
      devUrl: "http://localhost:5173",
      entry: "dist/index.html",
    },
    runtime: {
      type: "python",
      entry: "backend/main.py",
    },
    permissions: {
      filesystem: ["./data"],
      process: false,
      notifications: true,
      clipboard: true,
    },
    windows: [
      {
        id: "main",
        title: `${name} — Gateorix App`,
        width: 1200,
        height: 800,
      },
    ],
  };

  await fs.writeJson(path.join(targetDir, "gateorix.config.json"), manifest, {
    spaces: 2,
  });

  console.log(`  ✓ Project scaffolded at ./${name}`);
  console.log(`\n  Next steps:`);
  console.log(`    cd ${name}`);
  console.log(`    gateorix dev\n`);
}
