import type { DatabaseTable } from "@unfour/command-client";
import { describe, expect, it } from "vitest";
import { databaseTableTreeId } from "./database-tree";

function table(overrides: Partial<DatabaseTable>): DatabaseTable {
  return { columns: [], kind: "table", name: "users", ...overrides };
}

describe("databaseTableTreeId", () => {
  it("includes connection, schema, and table name", () => {
    expect(databaseTableTreeId("conn-1", table({ name: "orders", schema: "public" }))).toBe(
      "conn-1:table:public:orders",
    );
  });

  it("falls back to the default schema segment when schema is absent", () => {
    expect(databaseTableTreeId("conn-1", table({ schema: null }))).toBe(
      "conn-1:table:default:users",
    );
    expect(databaseTableTreeId("conn-1", table({ schema: undefined }))).toBe(
      "conn-1:table:default:users",
    );
  });
});
