/**
 * `gateorix add runtime <language>` — add a runtime adapter to the project.
 */

import path from "node:path";
import fs from "fs-extra";
import chalk from "chalk";

const SUPPORTED_RUNTIMES = ["python", "go", "dotnet", "swift", "cpp"];

const RUNTIME_CONFIG: Record<string, { type: string; entry: string }> = {
  python: { type: "python", entry: "backend/main.py" },
  go: { type: "go", entry: "backend/main.go" },
  dotnet: { type: "dotnet", entry: "backend/Program.cs" },
  swift: { type: "swift", entry: "backend/main.swift" },
  cpp: { type: "cpp", entry: "backend/main.cpp" },
};

/** Walk up from __dirname to find the package root (where examples/ lives). */
function findPackageRoot(): string {
  let dir = path.resolve(__dirname, "..");
  for (let i = 0; i < 10; i++) {
    if (fs.existsSync(path.join(dir, "examples"))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return path.resolve(__dirname, "..");
}

export async function addRuntimeCommand(language: string): Promise<void> {
  if (!SUPPORTED_RUNTIMES.includes(language)) {
    console.error(chalk.red(`\n  Error: unsupported runtime "${language}".`));
    console.error(`  Supported: ${SUPPORTED_RUNTIMES.join(", ")}\n`);
    process.exit(1);
  }

  const projectRoot = process.cwd();
  const configPath = path.join(projectRoot, "gateorix.config.json");

  if (!(await fs.pathExists(configPath))) {
    console.error(chalk.red("\n  No gateorix.config.json found in the current directory."));
    console.error(chalk.dim("  Run this command from your Gateorix project root.\n"));
    process.exit(1);
  }

  const config = await fs.readJson(configPath);
  if (config.runtime?.type) {
    console.error(chalk.yellow(`\n  Runtime already set to "${config.runtime.type}".`));
    console.error(chalk.dim("  Remove the runtime section from gateorix.config.json to change it.\n"));
    process.exit(1);
  }

  console.log(chalk.bold(`\n  Adding ${language} runtime adapter...\n`));

  // Copy backend files from matching example template
  const langMap: Record<string, string> = {
    python: "hello-react-python",
    go: "hello-react-go",
    dotnet: "hello-react-cs",
    cpp: "hello-react-cpp",
    swift: "hello-react-python", // fallback — swift template TBD
  };

  const pkgRoot = findPackageRoot();
  const templateBackend = path.join(pkgRoot, "examples", langMap[language], "backend");
  const targetBackend = path.join(projectRoot, "backend");

  if (await fs.pathExists(templateBackend)) {
    await fs.copy(templateBackend, targetBackend, { overwrite: false });
    console.log(`  ${chalk.green("✓")} Backend files copied to backend/`);
  } else {
    await fs.ensureDir(targetBackend);
    console.log(chalk.yellow(`  ! Template backend not found — created empty backend/ directory`));
  }

  // Update gateorix.config.json
  const runtimeConf = RUNTIME_CONFIG[language];
  config.runtime = runtimeConf;
  await fs.writeJson(configPath, config, { spaces: 2 });
  console.log(`  ${chalk.green("✓")} Updated gateorix.config.json`);

  // Update tauri.conf.json with externalBin sidecar config
  const tauriConfPath = path.join(projectRoot, "frontend", "src-tauri", "tauri.conf.json");
  if (await fs.pathExists(tauriConfPath)) {
    const tauriConf = await fs.readJson(tauriConfPath);

    // Determine sidecar binary name
    const sidecarNames: Record<string, string> = {
      python: "backend/main",
      go: "backend/hello-gateorix-go",
      dotnet: "backend/HelloGateorixCs",
      cpp: "backend/hello-gateorix-cpp",
      swift: "backend/hello-gateorix-swift",
    };
    const sidecarBin = sidecarNames[language] || `backend/${language}-sidecar`;

    // Add to bundle.externalBin
    if (!tauriConf.bundle) tauriConf.bundle = {};
    if (!tauriConf.bundle.externalBin) tauriConf.bundle.externalBin = [];
    if (!tauriConf.bundle.externalBin.includes(sidecarBin)) {
      tauriConf.bundle.externalBin.push(sidecarBin);
    }

    await fs.writeJson(tauriConfPath, tauriConf, { spaces: 2 });
    console.log(`  ${chalk.green("✓")} Updated tauri.conf.json with sidecar: ${sidecarBin}`);
  } else {
    console.log(chalk.dim("  • No src-tauri found — skipping tauri.conf.json update"));
  }

  // Print prerequisites
  const prereqs: Record<string, string> = {
    python: "Python ≥ 3.9 — pip install -r backend/requirements.txt",
    go: "Go ≥ 1.21 — cd backend && go mod tidy",
    dotnet: ".NET SDK ≥ 8.0 — cd backend && dotnet restore",
    cpp: "C++ compiler + CMake ≥ 3.16 — cd backend && cmake -B build && cmake --build build",
    swift: "Swift ≥ 5.9 — cd backend && swift build",
  };

  console.log(`\n  Prerequisites: ${chalk.dim(prereqs[language])}`);
  console.log(`  Then run: ${chalk.cyan("gateorix dev")}  ${chalk.dim("(or: gx dev)")}\n`);
}
