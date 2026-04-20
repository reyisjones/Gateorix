/**
 * `gateorix doctor` — check environment, dependencies, and project configuration.
 */

import { execSync } from "node:child_process";
import path from "node:path";
import fs from "fs-extra";
import chalk from "chalk";

interface CheckResult {
  name: string;
  status: "ok" | "warn" | "fail";
  detail: string;
  hint?: string;
}

function runCheck(name: string, command: string, minVersion?: string): CheckResult {
  try {
    const output = execSync(command, { encoding: "utf-8", stdio: ["pipe", "pipe", "pipe"] }).trim();
    if (minVersion) {
      const versionMatch = output.match(/(\d+\.\d+\.\d+)/);
      if (versionMatch) {
        const ver = versionMatch[1];
        if (compareVersions(ver, minVersion) < 0) {
          return { name, status: "warn", detail: ver, hint: `Minimum recommended: ${minVersion}` };
        }
      }
    }
    return { name, status: "ok", detail: output };
  } catch {
    return { name, status: "fail", detail: "not found", hint: `Install ${name} to continue` };
  }
}

function compareVersions(a: string, b: string): number {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    if ((pa[i] || 0) > (pb[i] || 0)) return 1;
    if ((pa[i] || 0) < (pb[i] || 0)) return -1;
  }
  return 0;
}

function icon(status: "ok" | "warn" | "fail"): string {
  if (status === "ok") return chalk.green("✓");
  if (status === "warn") return chalk.yellow("!");
  return chalk.red("✗");
}

async function checkProjectConfig(): Promise<CheckResult[]> {
  const results: CheckResult[] = [];
  const configPath = path.resolve(process.cwd(), "gateorix.config.json");

  if (await fs.pathExists(configPath)) {
    try {
      const config = await fs.readJson(configPath);
      results.push({ name: "gateorix.config.json", status: "ok", detail: `project "${config.name || "unnamed"}" v${config.version || "?"}` });

      // Check frontend entry
      if (config.frontend?.entry) {
        results.push({ name: "Frontend config", status: "ok", detail: `entry: ${config.frontend.entry}` });
      } else {
        results.push({ name: "Frontend config", status: "warn", detail: "no entry defined", hint: "Add frontend.entry to config" });
      }

      // Check runtime
      if (config.runtime?.type) {
        results.push({ name: "Runtime config", status: "ok", detail: `${config.runtime.type} → ${config.runtime.entry || "?"}` });
      } else {
        results.push({ name: "Runtime config", status: "warn", detail: "no runtime defined", hint: "Add runtime.type and runtime.entry" });
      }

      // Check permissions
      if (config.permissions) {
        const perms = Object.entries(config.permissions)
          .filter(([, v]) => v !== false)
          .map(([k]) => k);
        results.push({ name: "Permissions", status: "ok", detail: perms.length > 0 ? perms.join(", ") : "none granted" });
      }
    } catch {
      results.push({ name: "gateorix.config.json", status: "fail", detail: "invalid JSON", hint: "Fix the config file syntax" });
    }
  } else {
    results.push({ name: "gateorix.config.json", status: "warn", detail: "not found in current directory", hint: "Run `gateorix init <name>` to create a project" });
  }

  return results;
}

export async function doctorCommand(): Promise<void> {
  console.log(chalk.bold("\n  Gateorix Doctor\n"));

  // --- Toolchain checks ---
  console.log(chalk.bold("  Toolchain"));
  const toolchecks: CheckResult[] = [
    runCheck("Rust (rustc)", "rustc --version", "1.70.0"),
    runCheck("Cargo", "cargo --version", "1.70.0"),
    runCheck("Node.js", "node --version", "18.0.0"),
    runCheck("npm", "npm --version", "9.0.0"),
  ];

  // Python — try python3 first, then python (Windows)
  let pythonCheck = runCheck("Python", "python3 --version", "3.9.0");
  if (pythonCheck.status === "fail") {
    pythonCheck = runCheck("Python", "python --version", "3.9.0");
  }
  toolchecks.push(pythonCheck);

  // Go
  const goCheck = runCheck("Go", "go version", "1.21.0");
  toolchecks.push(goCheck);

  // .NET SDK
  const dotnetCheck = runCheck(".NET SDK", "dotnet --version", "8.0.0");
  toolchecks.push(dotnetCheck);

  // C++ compiler
  let cppCheck: CheckResult;
  if (process.platform === "win32") {
    cppCheck = runCheck("C++ Compiler (cl)", "cl 2>&1", undefined);
    if (cppCheck.status === "fail") {
      cppCheck = runCheck("C++ Compiler (g++)", "g++ --version");
      if (cppCheck.status === "fail") {
        cppCheck.hint = "Install Visual Studio Build Tools or MinGW";
      }
    }
  } else {
    cppCheck = runCheck("C++ Compiler (g++)", "g++ --version");
    if (cppCheck.status === "fail") {
      cppCheck = runCheck("C++ Compiler (clang++)", "clang++ --version");
    }
  }
  toolchecks.push(cppCheck);

  // CMake
  const cmakeCheck = runCheck("CMake", "cmake --version", "3.16.0");
  toolchecks.push(cmakeCheck);

  // Optional tools
  const tauriCheck = runCheck("Tauri CLI", "npx tauri --version");
  if (tauriCheck.status === "ok") {
    toolchecks.push(tauriCheck);
  }

  const gitCheck = runCheck("Git", "git --version");
  toolchecks.push(gitCheck);

  // WebView2 (Windows only)
  if (process.platform === "win32") {
    const webview2Check = runCheck(
      "WebView2",
      'reg query "HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" /v pv 2>nul',
    );
    if (webview2Check.status === "fail") {
      webview2Check.hint = "Install WebView2 Runtime from https://developer.microsoft.com/en-us/microsoft-edge/webview2/";
    }
    toolchecks.push(webview2Check);
  }

  // Xcode CLT (macOS only)
  if (process.platform === "darwin") {
    const xcodeCheck = runCheck("Xcode CLT", "xcode-select -p");
    if (xcodeCheck.status === "fail") {
      xcodeCheck.hint = "Run: xcode-select --install";
    }
    toolchecks.push(xcodeCheck);
  }

  for (const r of toolchecks) {
    const hint = r.hint ? chalk.dim(` (${r.hint})`) : "";
    console.log(`  ${icon(r.status)} ${r.name}: ${r.detail}${hint}`);
  }

  // --- Project config checks ---
  console.log(chalk.bold("\n  Project"));
  const configResults = await checkProjectConfig();
  for (const r of configResults) {
    const hint = r.hint ? chalk.dim(` (${r.hint})`) : "";
    console.log(`  ${icon(r.status)} ${r.name}: ${r.detail}${hint}`);
  }

  // --- Summary ---
  const all = [...toolchecks, ...configResults];
  const fails = all.filter((r) => r.status === "fail").length;
  const warns = all.filter((r) => r.status === "warn").length;

  console.log("");
  if (fails > 0) {
    console.log(chalk.red(`  ${fails} issue(s) found. Fix the errors above to use Gateorix.`));
  } else if (warns > 0) {
    console.log(chalk.yellow(`  ${warns} warning(s). Everything should work, but check the notes above.`));
  } else {
    console.log(chalk.green("  All checks passed! Your environment is ready."));
  }
  console.log("");
}
