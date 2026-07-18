// @vitest-environment jsdom
import type { ReactNode } from "react";
import { act, renderHook } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DatabaseTableWorkspaceTab } from "../model/types";

vi.mock("@unfour/command-client", () => ({ mutateDatabaseRow: vi.fn() }));

import { mutateDatabaseRow } from "@unfour/command-client";
import { useTableRowMutations } from "./useTableRowMutations";
import type { useDatabaseTabs } from "./useDatabaseTabs";

const mutateMock = vi.mocked(mutateDatabaseRow);

function createWrapper() {
  const client = new QueryClient({ defaultOptions: { mutations: { retry: false } } });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };
}

const tab: DatabaseTableWorkspaceTab = {
  connectionId: "conn-1",
  error: null,
  id: "table-1",
  kind: "table",
  pendingChanges: [
    {
      id: "update:1",
      operation: "update",
      originalValues: [{ column: "name", mode: "value", value: "Ada" }],
      primaryKey: [{ column: "id", mode: "value", value: "1" }],
      rowKey: "1",
      values: [{ column: "name", mode: "value", value: "Grace" }],
    },
  ],
  queryResult: null,
  segment: "data",
  structureTab: "ddl",
  table: { catalog: null, schema: null, name: "users", kind: "table", columns: [] },
  tableQuery: { filter: "", orderBy: null, orderDescending: false },
  tableView: { pageIndex: 0, pageSize: 50, readOnly: false, tableName: "users", totalRows: 1 },
  title: "users",
};

beforeEach(() => vi.clearAllMocks());

describe("useTableRowMutations", () => {
  it("applies confirmed changes with original values, then removes and refreshes", async () => {
    mutateMock.mockResolvedValue({ affectedRows: 1, sql: "UPDATE users SET name = ?" });
    const updateTableTab = vi.fn();
    const refreshTablePage = vi.fn();
    const databaseTabs = { updateTableTab } as unknown as ReturnType<typeof useDatabaseTabs>;
    const { result } = renderHook(
      () => useTableRowMutations({ activeTableTab: tab, databaseTabs, refreshTablePage, workspaceId: "ws-1" }),
      { wrapper: createWrapper() },
    );

    await act(() => result.current.applyPendingTableChanges());

    expect(mutateMock.mock.calls[0][0]).toEqual({
      workspaceId: "ws-1",
      connectionId: "conn-1",
      catalog: null,
      schema: null,
      tableName: "users",
      operation: "update",
      values: tab.pendingChanges[0].values,
      primaryKey: tab.pendingChanges[0].primaryKey,
      originalValues: tab.pendingChanges[0].originalValues,
      confirmMutation: true,
    });
    expect(updateTableTab).toHaveBeenCalledTimes(2);
    expect(refreshTablePage).toHaveBeenCalledOnce();
  });
});
