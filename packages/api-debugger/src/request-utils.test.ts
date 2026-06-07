import { describe, expect, it } from "vitest";
import {
  parseKeyValues,
  savedRequestToInput,
  groupSavedRequests,
  duplicateEnvironmentKeys,
  isSensitiveKey,
  formatByteSize,
  parseCollectionImport,
} from "./request-utils";
import type { ApiSavedRequest, KeyValue } from "@unfour/command-client";

describe("parseKeyValues", () => {
  it("parses valid JSON array", () => {
    const input = JSON.stringify([
      { key: "Content-Type", value: "application/json", enabled: true },
    ]);
    const result = parseKeyValues(input);
    expect(result).toHaveLength(1);
    expect(result[0].key).toBe("Content-Type");
  });

  it("returns empty array for invalid JSON", () => {
    expect(parseKeyValues("not json")).toEqual([]);
    expect(parseKeyValues("")).toEqual([]);
    expect(parseKeyValues("{")).toEqual([]);
  });

  it("filters out malformed items", () => {
    const input = JSON.stringify([
      { key: "valid", value: "ok", enabled: true },
      { key: "no-value", enabled: true },
      "not an object",
      42,
      null,
    ]);
    const result = parseKeyValues(input);
    expect(result).toHaveLength(1);
    expect(result[0].key).toBe("valid");
  });

  it("returns empty for non-array JSON", () => {
    expect(parseKeyValues('{"key":"val"}')).toEqual([]);
    expect(parseKeyValues('"string"')).toEqual([]);
    expect(parseKeyValues("123")).toEqual([]);
  });
});

describe("savedRequestToInput", () => {
  it("converts a saved request to input form", () => {
    const saved: ApiSavedRequest = {
      id: "req-1",
      workspaceId: "ws-1",
      name: "Get Users",
      folderPath: "Examples / Auth",
      method: "GET",
      url: "https://api.example.com/users",
      headersJson: JSON.stringify([
        { key: "Authorization", value: "Bearer token", enabled: true },
      ]),
      queryJson: "[]",
      body: null,
      bodyKind: "json",
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
      deletedAt: null,
      revision: 1,
      syncStatus: "local",
      remoteId: null,
    };
    const result = savedRequestToInput(saved, "ws-2");
    expect(result.workspaceId).toBe("ws-2");
    expect(result.name).toBe("Get Users");
    expect(result.method).toBe("GET");
    expect(result.headers).toHaveLength(1);
    expect(result.query).toEqual([]);
    expect(result.timeoutMs).toBe(60_000);
  });
});

describe("groupSavedRequests", () => {
  it("groups by folderPath with Unfiled first", () => {
    const items: ApiSavedRequest[] = [
      makeSavedRequest("B Request", "Beta"),
      makeSavedRequest("A Request", "Alpha"),
      makeSavedRequest("Unfiled One", ""),
      makeSavedRequest("Unfiled Two", null),
    ];
    const groups = groupSavedRequests(items);
    expect(groups).toHaveLength(3);
    expect(groups[0].folder).toBe("Unfiled");
    expect(groups[0].items).toHaveLength(2);
    expect(groups[1].folder).toBe("Alpha");
    expect(groups[2].folder).toBe("Beta");
  });

  it("sorts items within groups by name", () => {
    const items: ApiSavedRequest[] = [
      makeSavedRequest("Zebra", "Group"),
      makeSavedRequest("Apple", "Group"),
    ];
    const groups = groupSavedRequests(items);
    expect(groups[0].items[0].name).toBe("Apple");
    expect(groups[0].items[1].name).toBe("Zebra");
  });

  it("returns empty for no items", () => {
    expect(groupSavedRequests([])).toEqual([]);
  });
});

describe("duplicateEnvironmentKeys", () => {
  it("finds case-insensitive duplicate keys", () => {
    const vars: KeyValue[] = [
      { key: "API_URL", value: "a", enabled: true },
      { key: "api_url", value: "b", enabled: true },
      { key: "OTHER", value: "c", enabled: true },
    ];
    const dupes = duplicateEnvironmentKeys(vars);
    expect(dupes).toHaveLength(1);
    expect(dupes[0].toLowerCase()).toBe("api_url");
  });

  it("ignores disabled and empty keys", () => {
    const vars: KeyValue[] = [
      { key: "DUP", value: "a", enabled: true },
      { key: "dup", value: "b", enabled: false },
      { key: "", value: "c", enabled: true },
    ];
    expect(duplicateEnvironmentKeys(vars)).toEqual([]);
  });

  it("returns empty for no duplicates", () => {
    const vars: KeyValue[] = [
      { key: "A", value: "1", enabled: true },
      { key: "B", value: "2", enabled: true },
    ];
    expect(duplicateEnvironmentKeys(vars)).toEqual([]);
  });
});

describe("isSensitiveKey", () => {
  it("matches sensitive patterns", () => {
    expect(isSensitiveKey("authorization")).toBe(true);
    expect(isSensitiveKey("x-api-key")).toBe(true);
    expect(isSensitiveKey("Authorization")).toBe(true);
    expect(isSensitiveKey("secret_key")).toBe(true);
    expect(isSensitiveKey("password")).toBe(true);
    expect(isSensitiveKey("auth_token")).toBe(true);
    expect(isSensitiveKey("credential")).toBe(true);
  });

  it("does not match non-sensitive keys", () => {
    expect(isSensitiveKey("Content-Type")).toBe(false);
    expect(isSensitiveKey("Accept")).toBe(false);
    expect(isSensitiveKey("base_url")).toBe(false);
  });
});

describe("formatByteSize", () => {
  it("formats bytes correctly", () => {
    expect(formatByteSize(0)).toBe("0 B");
    expect(formatByteSize(512)).toBe("512 B");
    expect(formatByteSize(1023)).toBe("1023 B");
  });

  it("formats kilobytes", () => {
    expect(formatByteSize(1024)).toBe("1.0 KB");
    expect(formatByteSize(10240)).toBe("10 KB");
    expect(formatByteSize(51200)).toBe("50 KB");
  });

  it("formats megabytes", () => {
    expect(formatByteSize(1048576)).toBe("1.0 MB");
    expect(formatByteSize(10485760)).toBe("10 MB");
  });
});

describe("parseCollectionImport", () => {
  it("parses an array of requests", () => {
    const items = [
      { method: "GET", url: "https://example.com", name: "Test" },
    ];
    const result = parseCollectionImport(items, "ws-1");
    expect(result).toHaveLength(1);
    expect(result[0].method).toBe("GET");
    expect(result[0].workspaceId).toBe("ws-1");
  });

  it("parses object with savedRequests property", () => {
    const input = { savedRequests: [{ method: "POST", url: "https://api.test" }] };
    const result = parseCollectionImport(input, "ws-1");
    expect(result).toHaveLength(1);
    expect(result[0].method).toBe("POST");
  });

  it("filters invalid items", () => {
    const items = [
      { method: "GET", url: "https://valid.com" },
      "not an object",
      { method: 123, url: "bad" },
      null,
      { url: "missing method" },
    ];
    const result = parseCollectionImport(items, "ws-1");
    expect(result).toHaveLength(1);
  });

  it("returns empty for invalid input", () => {
    expect(parseCollectionImport(null, "ws-1")).toEqual([]);
    expect(parseCollectionImport("string", "ws-1")).toEqual([]);
    expect(parseCollectionImport(42, "ws-1")).toEqual([]);
  });
});

function makeSavedRequest(name: string, folderPath: string | null): ApiSavedRequest {
  return {
    id: `id-${name}`,
    workspaceId: "ws-1",
    name,
    folderPath,
    method: "GET",
    url: "https://example.com",
    headersJson: "[]",
    queryJson: "[]",
    body: null,
    bodyKind: "json",
    createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-01-01T00:00:00Z",
    deletedAt: null,
    revision: 1,
    syncStatus: "local",
    remoteId: null,
  };
}
