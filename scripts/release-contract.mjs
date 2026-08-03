#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const stableVersionPattern = /^\d+\.\d+\.\d+$/;

export function readWorkspaceVersion(repoRoot) {
  const cargo = readFileSync(resolve(repoRoot, "Cargo.toml"), "utf8");
  const workspaceBlock = cargo.match(
    /\[workspace\.package\][\s\S]*?(?:\r?\n\[|$)/,
  )?.[0];
  const version = workspaceBlock?.match(
    /^\s*version\s*=\s*"([^"]+)"\s*$/m,
  )?.[1];
  if (!version) throw new Error("Missing [workspace.package].version");
  return version;
}

export function validateCommunityStableRelease(version, tag) {
  if (!stableVersionPattern.test(version)) {
    throw new Error(
      `Community Stable requires workspace version X.Y.Z, got ${JSON.stringify(version)}`,
    );
  }
  const expectedTag = `v${version}`;
  if (tag !== expectedTag) {
    throw new Error(
      `Community Stable tag must exactly match ${expectedTag}, got ${JSON.stringify(tag)}`,
    );
  }
  return { version, tag, channel: "stable", prerelease: false };
}

export function resolveCommunityStableRelease(repoRoot, tag) {
  return validateCommunityStableRelease(readWorkspaceVersion(repoRoot), tag);
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : undefined;
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
    const result = resolveCommunityStableRelease(repoRoot, process.argv[2]);
    process.stdout.write(
      `version=${result.version}\ntag=${result.tag}\nchannel=${result.channel}\nprerelease=${result.prerelease}\n`,
    );
  } catch (error) {
    console.error(`[release-contract] ${error.message}`);
    process.exitCode = 1;
  }
}
