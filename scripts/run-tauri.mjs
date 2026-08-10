#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { dirname, extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  resolveTauriInvocation,
  tauriEnvironment,
} from "./release-environment.mjs";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const invocation = resolveTauriInvocation(process.argv.slice(2));
const { channel, environment } = tauriEnvironment(
  process.env,
  invocation.defaultChannel,
  invocation.forcedChannel,
);
const tauriArgs = invocation.args;
const args = ["--filter", "@unfour/desktop", "tauri", ...tauriArgs];

let command = "pnpm";
let commandArgs = args;
if (process.platform === "win32") {
  const pnpmEntry = process.env.npm_execpath;
  if (!pnpmEntry) {
    throw new Error(
      "On Windows, invoke this launcher through `pnpm tauri` so pnpm exposes its executable entry point",
    );
  }
  const extension = extname(pnpmEntry).toLowerCase();
  if ([".js", ".cjs", ".mjs"].includes(extension)) {
    command = process.execPath;
    commandArgs = [pnpmEntry, ...args];
  } else {
    command = pnpmEntry;
  }
}

console.log(`[run-tauri] Community release channel: ${channel}`);
const result = spawnSync(command, commandArgs, {
  cwd: repoRoot,
  stdio: "inherit",
  env: environment,
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);
