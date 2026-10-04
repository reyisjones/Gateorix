import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npm = process.platform === "win32" ? "npm.cmd" : "npm";

function run(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd, encoding: "utf8", timeout: 120000,
    shell: process.platform === "win32" && command === npm,
  });
  assert.equal(result.status, 0, `${command} ${args.join(" ")}\n${result.error || ""}\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
}

test("packed SDK exposes existing ESM and declaration entry points", { timeout: 180000 }, () => {
  const temporary = mkdtempSync(join(tmpdir(), "gateorix-sdk-"));
  try {
    mkdirSync(join(root, "dist"), { recursive: true });
    writeFileSync(join(root, "dist/index.js"), "throw new Error('stale build');");
    writeFileSync(join(root, "dist/stale.js"), "throw new Error('stale file');");
    const output = JSON.parse(run(npm, ["pack", "--json", "--pack-destination", temporary], root));
    const packed = Array.isArray(output) ? output[0] : Object.values(output)[0];
    const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
    const inventory = new Set(packed.files.map((file) => file.path));
    assert(!inventory.has("dist/stale.js"), "Prepack must remove stale build files");
    for (const field of ["main", "module", "types"]) {
      assert(inventory.has(manifest[field].replace(/^\.\//, "")), `Missing ${field} target: ${manifest[field]}`);
    }
    assert.equal(manifest.type, "module");
    assert.equal(manifest.exports["."].import, manifest.main);
    assert.equal(manifest.exports["."].types, manifest.types);
    for (const file of inventory) {
      assert(/^(?:dist\/.*\.(?:js|d\.ts)|dist\/LICENSE|package\.json|README\.md)$/.test(file), `Unexpected package file: ${file}`);
    }
    for (const required of ["README.md", "dist/LICENSE"]) assert(inventory.has(required));
    const consumer = join(temporary, "consumer");
    mkdirSync(consumer);
    writeFileSync(join(consumer, "package.json"), JSON.stringify({ private: true, type: "module" }));
    run(npm, ["install", "--ignore-scripts", "--no-audit", "--no-fund", join(temporary, packed.filename)], consumer);
    const installed = join(consumer, "node_modules", "@gateorix", "bridge");
    assert.equal(readFileSync(join(installed, "dist/LICENSE"), "utf8"), readFileSync(join(root, "../../LICENSE"), "utf8"));
    run(process.execPath, ["--input-type=module", "-e", "import { GateorixBridge } from '@gateorix/bridge'; if (typeof GateorixBridge !== 'function') throw Error('missing export'); new GateorixBridge();"], consumer);
    run(process.execPath, ["--input-type=commonjs", "-e", "const assert = require('node:assert/strict'); assert.throws(() => require('@gateorix/bridge'), { code: 'ERR_PACKAGE_PATH_NOT_EXPORTED' });"], consumer);
    run(process.execPath, ["--input-type=module", "-e", "import assert from 'node:assert/strict'; await assert.rejects(import('@gateorix/bridge/dist/bridge.js'), { code: 'ERR_PACKAGE_PATH_NOT_EXPORTED' });"], consumer);
    writeFileSync(join(consumer, "consumer.ts"), `import { GateorixBridge, type IpcRequest, type IpcResponse, type IpcEvent, type InvokeOptions } from '@gateorix/bridge';
const bridge = new GateorixBridge();
const request: IpcRequest = { id: 'test', channel: 'runtime.greet', payload: {} };
const options: InvokeOptions = { timeout: 100 };
const response: Promise<string> = bridge.invoke<string>(request.channel, request.payload, options);
const envelope: IpcResponse = { id: request.id, ok: true, payload: {} };
bridge.on('test', (event: IpcEvent) => { console.log(event, envelope, response); });
// @ts-expect-error timeout must be numeric
const invalid: InvokeOptions = { timeout: 'invalid' };
`);
    for (const [module, resolution] of [["NodeNext", "NodeNext"], ["ES2022", "Bundler"]]) {
      run(process.execPath, [join(root, "node_modules/typescript/bin/tsc"), "--noEmit", "--strict", "--target", "ES2022", "--module", module, "--moduleResolution", resolution, "consumer.ts"], consumer);
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});