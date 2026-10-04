import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { validateVersion, checkRegistry, digest, verifyDigest } from "../scripts/release-check.mjs";

const registry = { name: "@gateorixjs/cli", versions: { "0.3.2": {} }, "dist-tags": { latest: "0.3.2" } };

test("release version rejects duplicates, downgrades, prereleases and invalid metadata", () => {
  validateVersion("0.3.3", registry);
  validateVersion("0.10.0", registry);
  for (const version of ["0.3.2", "0.2.9", "0.4.0-beta.1", "01.0.0", "invalid"]) {
    assert.throws(() => validateVersion(version, registry));
  }
  assert.throws(() => validateVersion("0.3.3", {}));
  for (const versions of [[], "invalid", {}, null]) {
    assert.throws(() => validateVersion("0.3.3", { ...registry, versions }));
  }
});

test("registry errors fail closed", async () => {
  await checkRegistry("0.3.3", async () => ({ ok: true, json: async () => registry }));
  for (const status of [401, 404, 429, 500]) {
    await assert.rejects(checkRegistry("0.3.3", async () => ({ ok: false, status })));
  }
  await assert.rejects(checkRegistry("0.3.3", async () => { throw new Error("network unavailable"); }));
  await assert.rejects(checkRegistry("0.3.3", async () => ({ ok: true, json: async () => { throw new Error("invalid JSON"); } })));
});

test("artifact mutation prevents promotion", () => {
  const directory = mkdtempSync(join(tmpdir(), "gateorix-release-"));
  try {
    const file = join(directory, "package.tgz");
    writeFileSync(file, "original");
    const expected = digest(file);
    verifyDigest(file, expected);
    writeFileSync(file, "changed");
    assert.throws(() => verifyDigest(file, expected));
    assert.throws(() => verifyDigest(file, ""));
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});