#!/usr/bin/env node

/**
 * Gateorix CLI entry point.
 *
 * Commands:
 *   init <name>            Scaffold a new project from a template
 *   dev                    Start the app in development mode
 *   build                  Build the app for production
 *   doctor                 Check environment and dependencies
 *   add runtime <lang>     Add a runtime adapter
 *   add plugin <name>      Add a plugin
 */

import { Command } from "commander";
import { initCommand } from "./commands/init.js";
import { devCommand } from "./commands/dev.js";
import { buildCommand } from "./commands/build.js";
import { doctorCommand } from "./commands/doctor.js";
import { addRuntimeCommand } from "./commands/add-runtime.js";
import { addPluginCommand } from "./commands/add-plugin.js";

const program = new Command();

program
  .name("gateorix")
  .alias("gx")
  .description("Gateorix CLI — build cross-platform desktop apps with web UI and native power\n  Alias: gx (e.g. gx init, gx dev, gx build)")
  .version("0.2.0");

program
  .command("init <name>")
  .description("Scaffold a new Gateorix project")
  .option("-t, --template <template>", "starter template", "react-python")
  .action(initCommand);

program
  .command("dev")
  .description("Start the app in development mode with hot reload")
  .action(devCommand);

program
  .command("build")
  .description("Build the app for production")
  .option("--release", "optimized release build", false)
  .action(buildCommand);

program
  .command("doctor")
  .description("Check environment, dependencies, and configuration")
  .action(doctorCommand);

const addCmd = program.command("add").description("Add a runtime adapter or plugin");

addCmd
  .command("runtime <language>")
  .description("Add a runtime adapter (python, go, dotnet, swift, cpp)")
  .action(addRuntimeCommand);

addCmd
  .command("plugin <name>")
  .description("Add a plugin (filesystem, process, notifications, clipboard)")
  .action(addPluginCommand);

program.parse();
