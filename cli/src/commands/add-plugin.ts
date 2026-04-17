/**
 * `gateorix add plugin <name>` — add a plugin to the project.
 */

const AVAILABLE_PLUGINS = ["filesystem", "process", "notifications", "clipboard"];

export async function addPluginCommand(name: string): Promise<void> {
  if (!AVAILABLE_PLUGINS.includes(name)) {
    console.error(`\n  Error: unknown plugin "${name}".`);
    console.error(`  Available: ${AVAILABLE_PLUGINS.join(", ")}\n`);
    process.exit(1);
  }

  console.log(`\n  Adding ${name} plugin...`);

  // TODO: Implementation
  // 1. Update gateorix.config.json permissions
  // 2. Add plugin to the build dependency list

  console.log(`  [add plugin not yet implemented — coming in Phase 2]\n`);
}
