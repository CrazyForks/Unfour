import assert from "node:assert/strict";
import { mkdtemp, rm, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  classifyLineCount,
  countLines,
  formatIssue,
  scanLargeFiles,
  shouldSkipPath,
} from "./check-large-files.mjs";

function lines(count) {
  return Array.from({ length: count }, (_, index) => `line ${index + 1}`).join("\n");
}

test("classifyLineCount marks warning and error thresholds", () => {
  assert.equal(classifyLineCount(800), null);
  assert.deepEqual(classifyLineCount(801), {
    severity: "warning",
    threshold: 800,
  });
  assert.deepEqual(classifyLineCount(1201), {
    severity: "error",
    threshold: 1200,
  });
});

test("countLines handles empty and newline-terminated files", () => {
  assert.equal(countLines(""), 0);
  assert.equal(countLines("one"), 1);
  assert.equal(countLines("one\n"), 1);
  assert.equal(countLines("one\ntwo"), 2);
});

test("shouldSkipPath excludes generated, vendored, lock, and build output paths", () => {
  const root = path.join(tmpdir(), "unfour-large-file-test");

  assert.equal(shouldSkipPath(path.join(root, "node_modules/pkg/index.ts"), root), true);
  assert.equal(shouldSkipPath(path.join(root, "dist/index.js"), root), true);
  assert.equal(shouldSkipPath(path.join(root, "target/debug/lib.rs"), root), true);
  assert.equal(shouldSkipPath(path.join(root, "src/generated/types.ts"), root), true);
  assert.equal(shouldSkipPath(path.join(root, "src/client.generated.ts"), root), true);
  assert.equal(shouldSkipPath(path.join(root, "pnpm-lock.yaml"), root), true);
  assert.equal(shouldSkipPath(path.join(root, "src/app.ts"), root), false);
});

test("scanLargeFiles reports only supported source files over thresholds", async () => {
  const root = await mkdtemp(path.join(tmpdir(), "unfour-large-files-"));

  try {
    await mkdir(path.join(root, "src"), { recursive: true });
    await mkdir(path.join(root, "node_modules/pkg"), { recursive: true });
    await mkdir(path.join(root, "dist"), { recursive: true });
    await mkdir(path.join(root, "generated"), { recursive: true });

    await writeFile(path.join(root, "src", "warning.ts"), lines(801));
    await writeFile(path.join(root, "src", "error.tsx"), lines(1201));
    await writeFile(path.join(root, "src", "ok.rs"), lines(500));
    await writeFile(path.join(root, "src", "ignored.txt"), lines(1500));
    await writeFile(path.join(root, "node_modules/pkg/index.ts"), lines(2000));
    await writeFile(path.join(root, "dist/index.js"), lines(2000));
    await writeFile(path.join(root, "generated/types.ts"), lines(2000));

    const issues = await scanLargeFiles(root);

    assert.deepEqual(
      issues.map((issue) => ({
        path: issue.path,
        lineCount: issue.lineCount,
        severity: issue.severity,
        threshold: issue.threshold,
      })),
      [
        {
          path: "src/error.tsx",
          lineCount: 1201,
          severity: "error",
          threshold: 1200,
        },
        {
          path: "src/warning.ts",
          lineCount: 801,
          severity: "warning",
          threshold: 800,
        },
      ],
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("formatIssue includes actionable guidance", () => {
  const message = formatIssue({
    path: "src/large.tsx",
    lineCount: 900,
    severity: "warning",
    threshold: 800,
  });

  assert.match(message, /src\/large\.tsx/);
  assert.match(message, /900 lines/);
  assert.match(message, /threshold 800/);
  assert.match(message, /Extract types/);
});
