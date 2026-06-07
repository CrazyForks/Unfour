import { describe, expect, it } from "vitest";
import {
  serializeDatabaseResult,
  isConfirmationRequired,
  confirmationMessage,
  formatDatabaseError,
} from "./result-utils";
import type { DatabaseQueryResult } from "@unfour/command-client";

function makeResult(
  columns: string[],
  rows: (string | null)[][],
): DatabaseQueryResult {
  return {
    columns: columns.map((name) => ({ name, type: "TEXT" })),
    rows: rows as string[][],
    affectedRows: 0,
    safety: { classification: "read", confirmed: true },
    executionTimeMs: 1,
  };
}

describe("serializeDatabaseResult", () => {
  it("serializes a simple CSV result", () => {
    const result = makeResult(["id", "name"], [["1", "Alice"], ["2", "Bob"]]);
    const csv = serializeDatabaseResult(result, ",");
    expect(csv).toBe("id,name\r\n1,Alice\r\n2,Bob");
  });

  it("quotes cells containing delimiters", () => {
    const result = makeResult(["col"], [["hello, world"]]);
    const csv = serializeDatabaseResult(result, ",");
    expect(csv).toBe('col\r\n"hello, world"');
  });

  it("escapes double quotes within cells", () => {
    const result = makeResult(["col"], [['say "hello"']]);
    const csv = serializeDatabaseResult(result, ",");
    expect(csv).toBe('col\r\n"say ""hello"""');
  });

  it("quotes cells containing newlines", () => {
    const result = makeResult(["col"], [["line1\nline2"]]);
    const csv = serializeDatabaseResult(result, ",");
    expect(csv).toBe('col\r\n"line1\nline2"');
  });

  it("handles empty result set", () => {
    const result = makeResult(["id", "name"], []);
    const csv = serializeDatabaseResult(result, ",");
    expect(csv).toBe("id,name");
  });

  it("uses tab delimiter for TSV", () => {
    const result = makeResult(["a", "b"], [["1", "2"]]);
    const tsv = serializeDatabaseResult(result, "\t");
    expect(tsv).toBe("a\tb\r\n1\t2");
  });

  it("handles null values in rows", () => {
    const result = makeResult(["col"], [[null]]);
    const csv = serializeDatabaseResult(result, ",");
    expect(csv).toBe("col\r\n");
  });
});

describe("isConfirmationRequired", () => {
  it("returns true for matching error object", () => {
    expect(isConfirmationRequired({ code: "CONFIRMATION_REQUIRED" })).toBe(true);
  });

  it("returns false for other error codes", () => {
    expect(isConfirmationRequired({ code: "OTHER_ERROR" })).toBe(false);
  });

  it("returns false for non-objects", () => {
    expect(isConfirmationRequired(null)).toBe(false);
    expect(isConfirmationRequired(undefined)).toBe(false);
    expect(isConfirmationRequired("string")).toBe(false);
    expect(isConfirmationRequired(42)).toBe(false);
  });
});

describe("confirmationMessage", () => {
  it("includes classification when available", () => {
    const error = { code: "CONFIRMATION_REQUIRED", details: { classification: "write" } };
    const msg = confirmationMessage(error);
    expect(msg).toContain("write");
    expect(msg).toContain("Confirmation required");
  });

  it("returns fallback message when no details", () => {
    const msg = confirmationMessage({ code: "CONFIRMATION_REQUIRED" });
    expect(msg).toContain("Confirmation required");
  });

  it("returns fallback for non-object errors", () => {
    const msg = confirmationMessage(null);
    expect(msg).toContain("Confirmation required");
  });
});

describe("formatDatabaseError", () => {
  it("extracts Error message", () => {
    expect(formatDatabaseError(new Error("connection failed"))).toBe("connection failed");
  });

  it("returns string errors as-is", () => {
    expect(formatDatabaseError("disk full")).toBe("disk full");
  });

  it("returns default for unknown types", () => {
    expect(formatDatabaseError(null)).toBe("Unknown database error");
    expect(formatDatabaseError(42)).toBe("Unknown database error");
    expect(formatDatabaseError(undefined)).toBe("Unknown database error");
  });
});
