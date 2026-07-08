import { existsSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const migrationSets = [
  {
    label: "core",
    marker: "_core_",
    required: true,
    dir:
      process.env.UNFOUR_CORE_MIGRATIONS_DIR ??
      path.join(repoRoot, "crates/local-storage/migrations"),
  },
  {
    label: "pro",
    marker: "_pro_",
    required: process.env.UNFOUR_REQUIRE_PRO_MIGRATIONS === "1",
    dir:
      process.env.UNFOUR_PRO_MIGRATIONS_DIR ??
      path.resolve(repoRoot, "../unfour-pro/crates/pro-local-storage/migrations"),
  },
];

const timestampVersionPattern = /^\d{14}$/;
const seenVersions = new Map();
const errors = [];
const scanned = [];

for (const set of migrationSets) {
  if (!existsSync(set.dir)) {
    if (set.required) {
      errors.push(`${set.label}: migration directory does not exist: ${set.dir}`);
    }
    continue;
  }

  const files = readdirSync(set.dir)
    .filter((file) => file.endsWith(".sql"))
    .sort((a, b) => a.localeCompare(b));

  for (const file of files) {
    const firstUnderscore = file.indexOf("_");
    const version = firstUnderscore > 0 ? file.slice(0, firstUnderscore) : "";
    const entry = `${set.label}:${path.join(set.dir, file)}`;
    scanned.push(entry);

    if (!version || !/^\d+$/.test(version)) {
      errors.push(`${entry}: version must be pure digits before the first "_"`);
      continue;
    }

    if (!timestampVersionPattern.test(version)) {
      errors.push(
        `${entry}: version must be a YYYYMMDDHHMMSS timestamp, not local numbering`,
      );
    }

    if (!file.includes(set.marker)) {
      errors.push(`${entry}: filename must include ${set.marker}`);
    }

    const previous = seenVersions.get(version);
    if (previous) {
      errors.push(`${entry}: duplicate version ${version}; already used by ${previous}`);
    } else {
      seenVersions.set(version, entry);
    }
  }
}

if (errors.length > 0) {
  console.error("Migration check failed:");
  for (const error of errors) {
    console.error(`- ${error}`);
  }
  process.exit(1);
}

console.log(`Migration check passed (${scanned.length} files).`);
