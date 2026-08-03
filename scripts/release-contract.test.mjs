import assert from "node:assert/strict";
import test from "node:test";

import { validateCommunityStableRelease } from "./release-contract.mjs";

test("Community Stable accepts only the exact workspace version tag", () => {
  assert.deepEqual(validateCommunityStableRelease("1.2.3", "v1.2.3"), {
    version: "1.2.3",
    tag: "v1.2.3",
    channel: "stable",
    prerelease: false,
  });
  assert.throws(
    () => validateCommunityStableRelease("1.2.3", "v1.2.4"),
    /must exactly match v1\.2\.3/,
  );
});

test("pre-release tags cannot enter Community Stable", () => {
  for (const tag of ["v1.2.3-test.1", "v1.2.3-dev", "v1.2.3-rc.1"]) {
    assert.throws(
      () => validateCommunityStableRelease("1.2.3", tag),
      /must exactly match v1\.2\.3/,
    );
  }
  assert.throws(
    () => validateCommunityStableRelease("1.2.3-rc.1", "v1.2.3-rc.1"),
    /requires workspace version X\.Y\.Z/,
  );
});
