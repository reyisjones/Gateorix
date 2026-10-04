import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { once } from "node:events";
import { mkdtempSync, readFileSync, existsSync, rmSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const cliRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npm = process.platform === "win32" ? "npm.cmd" : "npm";

function run(command, args, cwd, input) {
  const result = spawnSync(command, args, {
    cwd, input, encoding: "utf8", timeout: 180000,
    shell: process.platform === "win32" && command === npm,
  });
  assert.equal(result.status, 0, `${command} ${args.join(" ")}\n${result.error || ""}\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
}

test("packed CLI scaffolds Rust independently and runs its backend", { timeout: 240000 }, async () => {
  const temporary = mkdtempSync(join(tmpdir(), "gateorix-rust-"));
  let server;
  let serverClosed;
  try {
    let tarball = process.env.GATEORIX_TEST_TARBALL;
    if (tarball) {
      tarball = resolve(tarball);
      assert(existsSync(tarball), `Missing release tarball: ${tarball}`);
    } else {
      const packed = JSON.parse(run(npm, ["pack", "--ignore-scripts", "--json", "--pack-destination", temporary], cliRoot));
      const metadata = Array.isArray(packed) ? packed[0] : Object.values(packed)[0];
      assert(metadata.files.some((file) => file.path === "templates/hello-react-rust/backend/gateorix-adapter/src/lib.rs"));
      tarball = join(temporary, metadata.filename);
    }
    const consumer = join(temporary, "consumer");
    mkdirSync(consumer);
    run(npm, ["install", "--ignore-scripts", "--no-audit", "--no-fund", tarball], consumer);
    const executable = join(consumer, "node_modules", "@gateorixjs", "cli", "dist", "index.js");
    for (const ui of ["react", "vue", "svelte", "solid", "vanilla"]) {
      run(process.execPath, [executable, "init", `${ui}-app`, "--template", `${ui}-rust`, "--no-install"], temporary);
      const project = join(temporary, `${ui}-app`);
      const config = JSON.parse(readFileSync(join(project, "gateorix.config.json"), "utf8"));
      assert.equal(config.runtime.type, "rust");
      assert.equal(config.runtime.entry, "backend/Cargo.toml");
      assert(existsSync(join(project, "backend/gateorix-adapter/src/lib.rs")));
      assert(!existsSync(join(project, "frontend/src-tauri")));
    }
    const existing = join(temporary, "existing");
    mkdirSync(existing);
    writeFileSync(join(existing, "gateorix.config.json"), JSON.stringify({ name: "existing", version: "0.1.0" }));
    run(process.execPath, [executable, "add", "runtime", "rust"], existing);
    assert(existsSync(join(existing, "backend/src/main.rs")));
    const project = join(temporary, "vanilla-app");
    run("cargo", ["build", "--manifest-path", "backend/Cargo.toml"], project);
    const binary = join(project, "backend/target/debug", process.platform === "win32" ? "gateorix-rust-backend.exe" : "gateorix-rust-backend");
    const request = JSON.stringify({ id: "smoke", channel: "runtime.greet", payload: { name: "Rust" } });
    const response = JSON.parse(run(binary, [], project, request + "\n"));
    assert.equal(response.id, "smoke");
    assert.equal(response.ok, true);
    assert.match(response.payload.message, /Hello, Rust/);
    server = spawn(binary, ["--http"], { cwd: project, stdio: ["ignore", "ignore", "pipe"] });
    serverClosed = once(server, "close");
    await new Promise((accept, reject) => {
      const deadline = setTimeout(() => reject(new Error("HTTP adapter did not start")), 10000);
      server.once("error", (error) => { clearTimeout(deadline); reject(error); });
      server.once("exit", (code) => { clearTimeout(deadline); reject(new Error(`HTTP adapter exited: ${code}`)); });
      server.stderr.on("data", (data) => {
        if (data.toString().includes("development adapter:")) { clearTimeout(deadline); accept(); }
      });
    });
    const invoke = await fetch("http://127.0.0.1:3001/invoke", {
      method: "POST", headers: { "Content-Type": "application/json", Origin: "http://localhost:5173" },
      body: request, signal: AbortSignal.timeout(5000),
    });
    assert.equal(invoke.status, 200);
    assert.equal((await invoke.json()).id, "smoke");
    const forbidden = await fetch("http://127.0.0.1:3001/invoke", {
      method: "POST", headers: { "Content-Type": "application/json", Origin: "https://untrusted.invalid" },
      body: request, signal: AbortSignal.timeout(5000),
    });
    assert.equal(forbidden.status, 403);
  } finally {
    if (server) {
      server.kill();
      await serverClosed;
    }
    rmSync(temporary, { recursive: true, force: true });
  }
});