import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import test from "node:test";

import {
  resolveReleaseChannel,
  tauriEnvironment,
} from "./release-environment.mjs";

test("local Tauri commands default to the Test release channel", () => {
  assert.equal(resolveReleaseChannel(undefined), "test");
  assert.equal(resolveReleaseChannel(""), "test");
});

test("Stable must be selected explicitly", () => {
  assert.equal(resolveReleaseChannel("stable"), "stable");
  assert.equal(resolveReleaseChannel(undefined), "test");
});

test("invalid release channels are rejected", () => {
  for (const value of ["dev", "nightly", "Stable", "test.1", " test", " "]) {
    assert.throws(
      () => resolveReleaseChannel(value),
      /must be exactly "test" or "stable"/,
    );
  }
});

test("the resolved channel reaches child processes", () => {
  const { environment } = tauriEnvironment({
    PATH: process.env.PATH,
    UNFOUR_RELEASE_CHANNEL: "test",
  });
  const child = spawnSync(
    process.execPath,
    ["-e", "process.stdout.write(process.env.UNFOUR_RELEASE_CHANNEL ?? '')"],
    { encoding: "utf8", env: environment },
  );

  assert.equal(child.status, 0);
  assert.equal(child.stdout, "test");
});
