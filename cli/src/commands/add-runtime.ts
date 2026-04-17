/**
 * `gateorix add runtime <language>` — add a runtime adapter to the project.
 */

const SUPPORTED_RUNTIMES = ["python", "go", "dotnet", "swift"];

export async function addRuntimeCommand(language: string): Promise<void> {
  if (!SUPPORTED_RUNTIMES.includes(language)) {
    console.error(
      `\n  Error: unsupported runtime "${language}".`
    );
    console.error(`  Supported: ${SUPPORTED_RUNTIMES.join(", ")}\n`);
    process.exit(1);
  }

  console.log(`\n  Adding ${language} runtime adapter...`);

  // TODO: Implementation
  // 1. Create backend/ directory with language-specific starter files
  // 2. Update gateorix.config.json with runtime configuration
  // 3. Install SDK for the chosen language

  console.log(`  [add runtime not yet implemented — coming in Phase 2]\n`);
}
