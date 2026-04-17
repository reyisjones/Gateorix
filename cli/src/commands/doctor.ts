/**
 * `gateorix doctor` — check environment and dependencies.
 */

import { execSync } from "node:child_process";

export async function doctorCommand(): Promise<void> {
  console.log("\n  Gateorix Doctor\n");

  const checks = [
    { name: "Rust (rustc)", command: "rustc --version" },
    { name: "Cargo", command: "cargo --version" },
    { name: "Node.js", command: "node --version" },
    { name: "npm", command: "npm --version" },
    { name: "Python", command: "python3 --version" },
  ];

  for (const check of checks) {
    try {
      const output = execSync(check.command, { encoding: "utf-8" }).trim();
      console.log(`  ✓ ${check.name}: ${output}`);
    } catch {
      console.log(`  ✗ ${check.name}: not found`);
    }
  }

  console.log("");
}
