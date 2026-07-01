import type { DatabaseWorkspaceTab } from "./types";
import { defaultSql } from "./database-state";

export const defaultDatabaseTabs: DatabaseWorkspaceTab[] = [
  {
    catalog: null,
    connectionId: null,
    error: null,
    id: "query",
    kind: "query",
    pendingConfirmation: false,
    result: null,
    resultTab: "results",
    schema: null,
    sql: defaultSql,
    title: "Query Console",
  },
];
