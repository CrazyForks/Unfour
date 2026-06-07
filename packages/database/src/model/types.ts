import type {
  DatabaseConnection,
  DatabaseQueryResult,
  DatabaseSchema,
  DatabaseTable,
} from "@unfour/command-client";

export type DatabaseTableViewState = {
  pageIndex: number;
  pageSize: number;
  readOnly: boolean;
  tableName: string;
  totalRows: number;
};

export type DatabaseWorkspaceTabKind = "sql" | "table-data" | "table-structure" | "view-data";

export type DatabaseWorkspaceTab = {
  connectionId?: string | null;
  id: string;
  kind: DatabaseWorkspaceTabKind;
  loading?: boolean;
  modified?: boolean;
  title: string;
};

export type DatabasePanelState = {
  connections: DatabaseConnection[];
  queryResult: DatabaseQueryResult | null;
  schema?: DatabaseSchema;
  selectedConnection: DatabaseConnection | null;
  selectedTable: DatabaseTable | null;
  tableView: DatabaseTableViewState | null;
};
