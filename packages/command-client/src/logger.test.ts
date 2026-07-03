import { describe, expect, it } from "vitest";
import { sanitizeLogValue } from "./logger";

describe("frontend logger sanitization", () => {
  it("redacts sensitive keys recursively", () => {
    const safe = sanitizeLogValue({
      authorization: "Bearer secret",
      nested: {
        private_key: "-----BEGIN PRIVATE KEY-----",
        license_key: "license-secret",
      },
      ok: "visible",
    });

    const text = JSON.stringify(safe);
    expect(text).toContain("<redacted>");
    expect(text).toContain("visible");
    expect(text).not.toContain("Bearer secret");
    expect(text).not.toContain("BEGIN PRIVATE KEY");
    expect(text).not.toContain("license-secret");
  });

  it("redacts token-like URL query parameters", () => {
    const safe = sanitizeLogValue({
      url: "https://api.example.test/items?access_token=secret&page=1&api_key=key",
    });

    const text = JSON.stringify(safe);
    expect(text).toContain("<redacted>");
    expect(text).toContain("page=1");
    expect(text).not.toContain("access_token=secret");
    expect(text).not.toContain("api_key=key");
  });
});
