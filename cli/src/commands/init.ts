/**
 * `gateorix init <name>` — scaffold a new Gateorix project.
 *
 * Interactive prompts for backend language and UI framework,
 * copies matching example template, replaces placeholders.
 */

import path from "node:path";
import { execSync } from "node:child_process";
import fs from "fs-extra";
import chalk from "chalk";
import { quickDoctorCheck } from "./doctor.js";

interface InitOptions {
  template?: string;
}

const BACKEND_LANGUAGES: Record<string, string> = {
  python: "python",
  go: "go",
  "c#": "cs",
  "f#": "fs",
  "c++": "cpp",
};

const UI_FRAMEWORKS = ["react", "vue", "svelte", "solid", "vanilla"];

/** Walk up from __dirname to find the package root (where examples/ lives). */
function findPackageRoot(): string {
  // When installed globally, templates ship alongside the CLI.
  // During dev they sit in the repo root.
  let dir = path.resolve(__dirname, "..");
  for (let i = 0; i < 10; i++) {
    if (fs.existsSync(path.join(dir, "examples"))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return path.resolve(__dirname, "..");
}

/** Replace placeholder project names inside every text file in a directory. */
async function replaceInDir(dir: string, oldName: string, newName: string): Promise<void> {
  const entries = await fs.readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules" || entry.name === "target" || entry.name === ".git") continue;
      await replaceInDir(full, oldName, newName);
    } else {
      try {
        const content = await fs.readFile(full, "utf-8");
        if (content.includes(oldName)) {
          await fs.writeFile(full, content.replaceAll(oldName, newName), "utf-8");
        }
      } catch {
        // skip binary files
      }
    }
  }
}

export async function initCommand(
  name: string,
  options: InitOptions
): Promise<void> {
  const targetDir = path.resolve(process.cwd(), name);

  if (await fs.pathExists(targetDir)) {
    console.error(chalk.red(`\n  Error: directory "${name}" already exists.\n`));
    process.exit(1);
  }

  // Quick environment check (warns but does not block)
  quickDoctorCheck();

  // Interactive prompts
  const inquirer = (await import("inquirer")).default;
  const answers = await inquirer.prompt([
    {
      type: "list",
      name: "language",
      message: "Backend language:",
      choices: Object.keys(BACKEND_LANGUAGES).map((k) => ({ name: k, value: k })),
      default: "python",
    },
    {
      type: "list",
      name: "ui",
      message: "UI framework:",
      choices: UI_FRAMEWORKS,
      default: "react",
    },
  ]);

  const langSlug = BACKEND_LANGUAGES[answers.language];
  const templateName = `hello-${answers.ui}-${langSlug}`;
  const pkgRoot = findPackageRoot();
  const templateDir = path.join(pkgRoot, "examples", templateName);

  if (!(await fs.pathExists(templateDir))) {
    console.error(chalk.red(`\n  Template not found: examples/${templateName}`));
    console.error(chalk.dim("  Available templates:"));
    try {
      const dirs = await fs.readdir(path.join(pkgRoot, "examples"));
      for (const d of dirs) console.error(chalk.dim(`    • ${d}`));
    } catch { /* ignore */ }
    process.exit(1);
  }

  console.log(chalk.bold(`\n  Creating Gateorix project: ${name}`));
  console.log(`  Backend : ${answers.language}`);
  console.log(`  UI      : ${answers.ui}`);
  console.log(`  Template: ${templateName}\n`);

  // Copy template
  await fs.copy(templateDir, targetDir, {
    filter: (src: string) => {
      const rel = path.relative(templateDir, src);
      return !rel.startsWith("node_modules") && !rel.startsWith("target") && !rel.includes(".git");
    },
  });

  // Replace placeholder names
  await replaceInDir(targetDir, templateName, name);

  // Update gateorix.config.json with actual project name
  const configPath = path.join(targetDir, "gateorix.config.json");
  if (await fs.pathExists(configPath)) {
    const config = await fs.readJson(configPath);
    config.name = name;
    config.ui = answers.ui;
    await fs.writeJson(configPath, config, { spaces: 2 });
  }

  // Install frontend dependencies
  const frontendDir = path.join(targetDir, "frontend");
  if (await fs.pathExists(path.join(frontendDir, "package.json"))) {
    console.log(`  ${chalk.cyan("→")} Installing frontend dependencies...`);
    try {
      execSync("npm install", { cwd: frontendDir, stdio: "pipe" });
      console.log(`  ${chalk.green("✓")} Dependencies installed`);
    } catch {
      console.log(chalk.yellow("  ! npm install failed — run it manually after setup"));
    }
  }

  console.log(chalk.green(`\n  ✓ Project scaffolded at ./${name}\n`));
  console.log(`  Next steps:`);
  console.log(`    ${chalk.cyan("cd")} ${name}`);
  console.log(`    ${chalk.cyan("gateorix dev")}  ${chalk.dim("(or: gx dev)")}\n`);
}
