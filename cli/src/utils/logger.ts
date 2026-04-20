/**
 * Color-coded logging helpers for the CLI.
 */

import chalk from "chalk";

export const log = {
  /** Bold heading */
  heading(text: string): void {
    console.log(chalk.bold(`\n  ${text}\n`));
  },

  /** Step indicator (→) */
  step(text: string): void {
    console.log(`  ${chalk.cyan("→")} ${text}`);
  },

  /** Success (✓) */
  success(text: string): void {
    console.log(`  ${chalk.green("✓")} ${text}`);
  },

  /** Warning (!) */
  warn(text: string): void {
    console.log(`  ${chalk.yellow("!")} ${text}`);
  },

  /** Error (✗) */
  error(text: string): void {
    console.error(chalk.red(`\n  ${text}\n`));
  },

  /** Dimmed info text */
  dim(text: string): void {
    console.log(chalk.dim(`  ${text}`));
  },

  /** Blank line */
  blank(): void {
    console.log("");
  },
};
