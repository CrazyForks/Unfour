import type {
  DatabaseCellValue,
  DatabaseConnection,
  DatabaseQueryResult,
  DatabaseSchema,
  DatabaseTable,
} from "@unfour/command-client";

export type TableEditing = {
  pending: boolean;
  primaryKeyColumns: string[];
  onDeleteRow: (primaryKey: DatabaseCellValue[]) => void;
  onInsertRow: (values: DatabaseCellValue[]) => void;
  onUpdateCell: (columnName: string, value: string | null, primaryKey: DatabaseCellValue[]) => void;
};

export type DatabaseTableViewState = {
  pageIndex: number;
  pageSize: number;
  readOnly: boolean;
  tableName: string;
  totalRows: number;
};

export type DatabaseConnectionStatus = "disconnected" | "connecting" | "connected" | "failed";

export type DatabaseConnectionSessionState = {
  message?: string | null;
  serverVersion?: string | null;
  status: DatabaseConnectionStatus;
  updatedAt?: string;
};

export type SqlHistoryEntry = {
  affectedRows?: number;
  classification?: string;
  connectionId: string | null;
  connectionName: string;
  durationMs?: number;
  error?: string;
  executedAt: string;
  id: string;
  rowCount?: number;
  sql: string;
  status: "success" | "failed";
};

// Explicit execution context for a query window. `connectionId` identifies the
// datasource; `catalog`/`schema` scope where unqualified names resolve and which
// database the statement runs against (applied server-side before execution).
export type QueryContext = {
  connectionId: string | null;
  catalog: string | null;
  schema: string | null;
};

export type DatabaseResultTab = "results" | "messages" | "logs" | "history";

export type DatabaseStructureTab = "ddl" | "indexes" | "constraints" | "properties";

export type TableQueryState = {
  orderBy: string | null;
  orderDescending: boolean;
  filter: string;
};

export const emptyTableQuery: TableQueryState = {
  orderBy: null,
  orderDescending: false,
  filter: "",
};

// Object-level workspace tab ids are dynamic so multiple query/table objects
// can stay open at once.
export type DatabaseWorkspaceTabId = string;

export type TableSegment = "data" | "structure";

export type DatabaseWorkspaceTabKind = "query" | "table";

export type DatabaseQueryWorkspaceTab = {
  catalog: string | null;
  connectionId: string | null;
  error: unknown;
  id: DatabaseWorkspaceTabId;
  kind: "query";
  loading?: boolean;
  pendingConfirmation: boolean;
  result: DatabaseQueryResult | null;
  resultTab: DatabaseResultTab;
  schema: string | null;
  sql: string;
  title: string;
};

export type DatabaseTableWorkspaceTab = {
  connectionId: string;
  error: unknown;
  id: DatabaseWorkspaceTabId;
  kind: "table";
  loading?: boolean;
  queryResult: DatabaseQueryResult | null;
  segment: TableSegment;
  structureTab: DatabaseStructureTab;
  table: DatabaseTable;
  tableQuery: TableQueryState;
  tableView: DatabaseTableViewState | null;
  title: string;
};

export type DatabaseWorkspaceTab = DatabaseQueryWorkspaceTab | DatabaseTableWorkspaceTab;

export type DatabasePanelState = {
  connections: DatabaseConnection[];
  queryResult: DatabaseQueryResult | null;
  schema?: DatabaseSchema;
  selectedConnection: DatabaseConnection | null;
  selectedTable: DatabaseTable | null;
  tableView: DatabaseTableViewState | null;
};
