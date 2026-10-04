import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export function validateVersion(version, registry) {
  const stable = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
  if (typeof version !== "string" || !stable.test(version)) throw new Error("Only stable x.y.z releases to latest are supported.");
  const latest = registry?.["dist-tags"]?.latest;
  if (registry?.name !== "@gateorixjs/cli" || !registry.versions ||
      typeof registry.versions !== "object" || Array.isArray(registry.versions) ||
      typeof latest !== "string" || !stable.test(latest) || !Object.hasOwn(registry.versions, latest)) {
    throw new Error("Invalid registry metadata; refusing release.");
  }
  if (Object.hasOwn(registry.versions, version)) throw new Error(`Version ${version} already exists.`);
  const current = latest.split(".").map(BigInt);
  const candidate = version.split(".").map(BigInt);
  const difference = candidate.findIndex((value, index) => value !== current[index]);
  if (difference < 0 || candidate[difference] < current[difference]) throw new Error("Version must be newer than latest.");
}

export async function checkRegistry(version, fetcher = fetch) {
  const response = await fetcher("https://registry.npmjs.org/@gateorixjs%2fcli", {
    signal: AbortSignal.timeout(15000),
  });
  if (!response.ok) throw new Error(`Registry HTTP ${response.status}; refusing release.`);
  validateVersion(version, await response.json());
}

export function digest(file) {
  return createHash("sha512").update(readFileSync(file)).digest("hex");
}

export function verifyDigest(file, expected) {
  if (!/^[a-f0-9]{128}$/.test(expected) || digest(file) !== expected) {
    throw new Error("Artifact digest mismatch; refusing release.");
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const [operation, value, expected] = process.argv.slice(2);
    if (operation === "version") {
      const manifest = JSON.parse(readFileSync(new URL("../package.json", import.meta.url)));
      if (manifest.name !== "@gateorixjs/cli" || manifest.version !== value) throw new Error("Requested version does not match CLI manifest.");
      await checkRegistry(value);
      console.log(`Release candidate ${value} is available.`);
    } else if (operation === "digest") {
      console.log(digest(value));
    } else if (operation === "verify") {
      verifyDigest(value, expected);
      console.log("Artifact digest verified.");
    } else {
      throw new Error("Usage: release-check.mjs version VERSION | digest FILE | verify FILE SHA512");
    }
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}