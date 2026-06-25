import { Columns3, Copy, Database, Eye, MoreHorizontal, Pencil, Play, RefreshCw, Table2, Trash2 } from "lucide-react";
import type { DatabaseConnection, DatabaseSchema, DatabaseTable } from "@unfour/command-client";
import {
  Badge,
  ConnectionStatus,
  ContextMenuItem,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  EmptyState,
  IconButton,
  TreeView,
  useI18n,
  type TreeViewItem,
} from "@unfour/ui";
import { buildDatabaseTree, databaseTableTreeId } from "../model/database-tree";
import type { DatabaseConnectionSessionState, DatabaseConnectionStatus } from "../model/types";

export function DatabaseConnectionTree({
  catalogs,
  connectionStates,
  connections,
  loadingCatalogs,
  onConnect,
  onDeleteConnection,
  onDisconnect,
  onEditConnection,
  onNewQuery,
  onPreviewTable,
  onRefresh,
  onRefreshSchema,
  onSelectConnection,
  onSelectTable,
  onToggleCatalog,
  onUseSql,
  schema,
  schemaLoading = false,
  schemasByCatalog,
  selectedConnectionId,
  selectedTableId,
}: {
  /** Server databases (catalogs) for the selected connection. Empty/undefined
   * for SQLite, which has no catalog level. */
  catalogs?: string[];
  connectionStates?: Record<string, DatabaseConnectionSessionState>;
  connections: DatabaseConnection[];
  /** Catalogs whose schema fetch is currently in flight. */
  loadingCatalogs?: string[];
  onConnect?: (connection: DatabaseConnection) => void;
  onDeleteConnection?: (connection: DatabaseConnection) => void;
  onDisconnect?: (connection: DatabaseConnection) => void;
  onEditConnection?: (connection: DatabaseConnection) => void;
  onNewQuery?: () => void;
  onPreviewTable?: (table: DatabaseTable) => void;
  onRefresh?: () => void;
  onRefreshSchema?: (connection: DatabaseConnection) => void;
  onSelectConnection: (connection: DatabaseConnection) => void;
  onSelectTable?: (table: DatabaseTable) => void;
  /** Fired when a catalog node is expanded, so its schema can be lazy-loaded. */
  onToggleCatalog?: (catalog: string) => void;
  onUseSql?: (sql: string) => void;
  schema?: DatabaseSchema;
  schemaLoading?: boolean;
  /** Per-catalog loaded schemas for the selected connection. */
  schemasByCatalog?: Record<string, DatabaseSchema>;
  selectedConnectionId: string | null;
  selectedTableId?: string | null;
}) {
  const { t } = useI18n();

  if (!connections.length) {
    return <EmptyState className="min-h-[72px]">{t("database.errors.noConnections")}</EmptyState>;
  }

  const tableLookup = new Map<string, DatabaseTable>();
  // Maps a catalog node id to its catalog name so expansion can trigger a
  // lazy schema load for that database.
  const catalogLookup = new Map<string, string>();
  const defaultExpandedIds = new Set<string>();
  const selectedConnection = connections.find((connection) => connection.id === selectedConnectionId) ?? null;

  const items: TreeViewItem[] = connections.map((connection) => {
    const selected = connection.id === selectedConnectionId;
    const session = connectionStates?.[connection.id];
    const status = resolveConnectionStatus({
      hasSchema: selected && Boolean(schema),
      selected,
      session,
    });

    if (selected) {
      defaultExpandedIds.add(connection.id);
    }

    return {
      actions: (
        <ConnectionActions
          connection={connection}
          onConnect={onConnect}
          onDeleteConnection={onDeleteConnection}
          onDisconnect={onDisconnect}
          onEditConnection={onEditConnection}
          onNewQuery={onNewQuery}
          onRefresh={onRefresh}
          onRefreshSchema={onRefreshSchema}
          status={status}
        />
      ),
      contextMenu: (
        <ConnectionContextMenu
          connection={connection}
          onConnect={onConnect}
          onDeleteConnection={onDeleteConnection}
          onDisconnect={onDisconnect}
          onEditConnection={onEditConnection}
          onNewQuery={onNewQuery}
          onRefreshSchema={onRefreshSchema}
          status={status}
        />
      ),
      children: selected
        ? buildSelectedConnectionChildren({
            catalogLookup,
            catalogs,
            connection,
            defaultExpandedIds,
            loadingCatalogs,
            onPreviewTable,
            onRefreshSchema,
            onUseSql,
            schema,
            schemaLoading,
            schemasByCatalog,
            status,
            t,
            tableLookup,
          })
        : undefined,
      icon: <Database size={13} />,
      id: connection.id,
      label: connection.name,
      meta: (
        <ConnectionStatus
          label={status}
          status={status === "failed" ? "error" : status}
        />
      ),
      title: connectionStateTitle(connection, session),
    };
  });

  const selectedId = selectedTableId ?? selectedConnection?.id ?? null;

  return (
    <TreeView
      key={[selectedConnectionId, schema?.tables.length ?? 0, schemaLoading ? "loading" : "idle"].join(":")}
      defaultExpandedIds={[...defaultExpandedIds]}
      items={items}
      onSelect={(item) => {
        const table = tableLookup.get(item.id);
        if (table) {
          onSelectTable?.(table);
          return;
        }

        const connection = connections.find((candidate) => candidate.id === item.id);
        if (connection) {
          onSelectConnection(connection);
        }
      }}
      onToggle={(id, expanded) => {
        const catalog = catalogLookup.get(id);
        if (catalog && expanded) {
          onToggleCatalog?.(catalog);
        }
      }}
      selectedId={selectedId}
    />
  );
}

function buildSelectedConnectionChildren({
  catalogLookup,
  catalogs,
  connection,
  defaultExpandedIds,
  loadingCatalogs,
  onPreviewTable,
  onRefreshSchema,
  onUseSql,
  schema,
  schemaLoading,
  schemasByCatalog,
  status,
  t,
  tableLookup,
}: {
  catalogLookup: Map<string, string>;
  catalogs?: string[];
  connection: DatabaseConnection;
  defaultExpandedIds: Set<string>;
  loadingCatalogs?: string[];
  onPreviewTable?: (table: DatabaseTable) => void;
  onRefreshSchema?: (connection: DatabaseConnection) => void;
  onUseSql?: (sql: string) => void;
  schema?: DatabaseSchema;
  schemaLoading: boolean;
  schemasByCatalog?: Record<string, DatabaseSchema>;
  status: DatabaseConnectionStatus;
  t: ReturnType<typeof useI18n>["t"];
  tableLookup: Map<string, DatabaseTable>;
}): TreeViewItem[] {
  if (status === "disconnected") {
    return [
      {
        disabled: true,
        id: `${connection.id}:disconnected`,
        label: "Connect to browse schema",
      },
    ];
  }

  if (status === "failed") {
    return [
      {
        disabled: true,
        id: `${connection.id}:failed`,
        label: "Connection failed",
      },
    ];
  }

  // Multi-database (catalog) mode for PostgreSQL/MySQL: every server database
  // is a node. PostgreSQL cannot cross-database query, so each database's schema
  // is loaded lazily when its node is expanded; the connected database (and all
  // MySQL databases) arrive in the initial schema load.
  if (catalogs && catalogs.length > 0) {
    // Index every catalog's tables: the connected database (and, for MySQL,
    // every database) comes from the initial schema; lazily-loaded databases
    // come from schemasByCatalog.
    const tablesByCatalog = new Map<string, DatabaseTable[]>();
    for (const table of schema?.tables ?? []) {
      const key = table.catalog ?? "";
      tablesByCatalog.set(key, [...(tablesByCatalog.get(key) ?? []), table]);
    }
    for (const [name, catalogSchema] of Object.entries(schemasByCatalog ?? {})) {
      tablesByCatalog.set(name, catalogSchema.tables);
    }

    const connectedCatalog = connection.database?.trim() || null;

    return catalogs.map((name) => {
      const catalogNodeId = `${connection.id}:catalog:${name}`;
      catalogLookup.set(catalogNodeId, name);
      const tables = tablesByCatalog.get(name);
      const loading =
        (loadingCatalogs?.includes(name) ?? false) ||
        (name === connectedCatalog && schemaLoading && !tables);

      let children: TreeViewItem[];
      if (tables) {
        children = renderCatalogContents({
          connection,
          defaultExpandedIds,
          onPreviewTable,
          onRefreshSchema,
          onUseSql,
          parentId: catalogNodeId,
          t,
          tableLookup,
          tables,
        });
        // Auto-expand the connected database; others stay collapsed until the
        // user opens them.
        if (name === connectedCatalog) {
          defaultExpandedIds.add(catalogNodeId);
        }
      } else {
        children = [
          {
            disabled: true,
            id: `${catalogNodeId}:${loading ? "loading" : "placeholder"}`,
            label: loading ? t("database.tree.loadingSchema") : t("database.tree.expandToLoad"),
          },
        ];
      }

      return {
        children,
        icon: <Database size={13} />,
        id: catalogNodeId,
        label: name,
        title: name,
      };
    });
  }

  if (schemaLoading) {
    return [
      {
        disabled: true,
        id: `${connection.id}:loading`,
        label: "Loading schema...",
      },
    ];
  }

  if (!schema) {
    return [
      {
        disabled: true,
        id: `${connection.id}:schema-empty`,
        label: "Schema not loaded",
      },
    ];
  }

  if (!schema.tables.length) {
    return [
      {
        disabled: true,
        id: `${connection.id}:no-tables`,
        label: "No tables or views found",
      },
    ];
  }

  // Single-datasource mode (SQLite): one file node containing its objects.
  const model = buildDatabaseTree(schema.tables);

  return model.catalogs.map((catalog) => {
    const catalogLabel = catalog.key || databaseLabel(connection);
    const catalogNodeId = `${connection.id}:catalog:${catalog.key || "default"}`;
    defaultExpandedIds.add(catalogNodeId);
    const totalTables = catalog.schemas.reduce((sum, node) => sum + node.tables.length, 0);
    return {
      children: renderCatalogContents({
        connection,
        defaultExpandedIds,
        onPreviewTable,
        onRefreshSchema,
        onUseSql,
        parentId: catalogNodeId,
        t,
        tableLookup,
        tables: catalog.schemas.flatMap((node) => node.tables),
      }),
      icon: <Database size={13} />,
      id: catalogNodeId,
      label: catalogLabel,
      meta: <Badge tone="neutral">{totalTables}</Badge>,
      title: catalogLabel,
    };
  });
}

// Render the contents of a single catalog (database): PostgreSQL nests schemas,
// while MySQL/SQLite list table groups directly. Shared by the lazy multi-catalog
// renderer and the single-datasource (SQLite) renderer.
function renderCatalogContents({
  connection,
  defaultExpandedIds,
  onPreviewTable,
  onRefreshSchema,
  onUseSql,
  parentId,
  t,
  tableLookup,
  tables,
}: {
  connection: DatabaseConnection;
  defaultExpandedIds: Set<string>;
  onPreviewTable?: (table: DatabaseTable) => void;
  onRefreshSchema?: (connection: DatabaseConnection) => void;
  onUseSql?: (sql: string) => void;
  parentId: string;
  t: ReturnType<typeof useI18n>["t"];
  tableLookup: Map<string, DatabaseTable>;
  tables: DatabaseTable[];
}): TreeViewItem[] {
  if (!tables.length) {
    return [
      {
        disabled: true,
        id: `${parentId}:no-tables`,
        label: "No tables or views found",
      },
    ];
  }

  const catalog = buildDatabaseTree(tables).catalogs[0];
  if (!catalog || !catalog.hasSchemaLevel) {
    return buildTableGroups({
      connection,
      defaultExpandedIds,
      onPreviewTable,
      onUseSql,
      parentId,
      t,
      tableLookup,
      tables,
    });
  }

  return catalog.schemas.map((schemaNode) => {
    const schemaNodeId = `${parentId}:schema:${schemaNode.key}`;
    defaultExpandedIds.add(schemaNodeId);
    return {
      actions: onRefreshSchema ? (
        <IconButton label={`Refresh ${schemaNode.key} schema`} onClick={() => onRefreshSchema(connection)} size="compact">
          <RefreshCw size={12} />
        </IconButton>
      ) : undefined,
      children: buildTableGroups({
        connection,
        defaultExpandedIds,
        onPreviewTable,
        onUseSql,
        parentId: schemaNodeId,
        t,
        tableLookup,
        tables: schemaNode.tables,
      }),
      icon: <Columns3 size={13} />,
      id: schemaNodeId,
      label: schemaNode.key,
      meta: <Badge tone="neutral">{schemaNode.tables.length}</Badge>,
      title: schemaNode.key,
    };
  });
}

// Categorize a schema's objects into Tables and Views group nodes (each with a
// count badge), matching the asset-tree hierarchy in the design mockup. The
// caller has already guarded against the empty-schema case.
function buildTableGroups({
  connection,
  defaultExpandedIds,
  onPreviewTable,
  onUseSql,
  parentId,
  t,
  tableLookup,
  tables,
}: {
  connection: DatabaseConnection;
  defaultExpandedIds: Set<string>;
  onPreviewTable?: (table: DatabaseTable) => void;
  onUseSql?: (sql: string) => void;
  parentId: string;
  t: ReturnType<typeof useI18n>["t"];
  tableLookup: Map<string, DatabaseTable>;
  tables: DatabaseTable[];
}): TreeViewItem[] {
  const baseTables = tables.filter((table) => !isViewKind(table.kind));
  const views = tables.filter((table) => isViewKind(table.kind));
  const groups: TreeViewItem[] = [];

  if (baseTables.length) {
    const groupId = `${parentId}:tables`;
    defaultExpandedIds.add(groupId);
    groups.push({
      children: baseTables.map((table) =>
        tableItem({ connection, onPreviewTable, onUseSql, t, table, tableLookup }),
      ),
      icon: <Table2 size={13} />,
      id: groupId,
      label: t("database.tree.tablesGroup"),
      meta: <Badge tone="teal">{baseTables.length}</Badge>,
    });
  }

  if (views.length) {
    const groupId = `${parentId}:views`;
    groups.push({
      children: views.map((table) =>
        tableItem({ connection, onPreviewTable, onUseSql, t, table, tableLookup }),
      ),
      icon: <Eye size={13} />,
      id: groupId,
      label: t("database.tree.viewsGroup"),
      meta: <Badge tone="neutral">{views.length}</Badge>,
    });
  }

  return groups;
}

function isViewKind(kind: string) {
  return kind.toLowerCase().includes("view");
}

function tableItem({
  connection,
  onPreviewTable,
  onUseSql,
  t,
  table,
  tableLookup,
}: {
  connection: DatabaseConnection;
  onPreviewTable?: (table: DatabaseTable) => void;
  onUseSql?: (sql: string) => void;
  t: ReturnType<typeof useI18n>["t"];
  table: DatabaseTable;
  tableLookup: Map<string, DatabaseTable>;
}): TreeViewItem {
  const id = databaseTableTreeId(connection.id, table);
  tableLookup.set(id, table);
  return {
    actions: onPreviewTable ? (
      <IconButton label={`Open preview for ${table.name}`} onClick={() => onPreviewTable(table)} size="compact">
        <Play size={12} />
      </IconButton>
    ) : undefined,
    contextMenu: (
      <TableContextMenu
        connection={connection}
        onPreviewTable={onPreviewTable}
        onUseSql={onUseSql}
        t={t}
        table={table}
      />
    ),
    children: table.columns.map((column) => ({
      icon: <Columns3 size={12} />,
      id: `${id}:column:${column.name}`,
      label: column.name,
      meta: column.primaryKey ? <Badge tone="green">PK</Badge> : undefined,
      title: `${column.name} ${column.dataType}`,
    })),
    icon: <Table2 size={13} />,
    id,
    label: table.name,
    meta: <Badge tone="neutral">{table.kind}</Badge>,
    title: [table.catalog, table.schema, table.name].filter(Boolean).join("."),
  };
}

function TableContextMenu({
  connection,
  onPreviewTable,
  onUseSql,
  t,
  table,
}: {
  connection: DatabaseConnection;
  onPreviewTable?: (table: DatabaseTable) => void;
  onUseSql?: (sql: string) => void;
  t: ReturnType<typeof useI18n>["t"];
  table: DatabaseTable;
}) {
  return (
    <>
      {onPreviewTable && (
        <ContextMenuItem onSelect={() => onPreviewTable(table)}>
          {t("database.tree.previewData")}
        </ContextMenuItem>
      )}
      {onUseSql && (
        <ContextMenuItem onSelect={() => onUseSql(generateSelectSql(connection.driver, table))}>
          {t("database.tree.generateSelect")}
        </ContextMenuItem>
      )}
      {onUseSql && (
        <ContextMenuItem onSelect={() => onUseSql(generateInsertSql(connection.driver, table))}>
          {t("database.tree.generateInsert")}
        </ContextMenuItem>
      )}
      <ContextMenuItem onSelect={() => void navigator.clipboard?.writeText(table.name)}>
        {t("database.tree.copyTableName")}
      </ContextMenuItem>
    </>
  );
}

function quoteDbIdentifier(driver: string, value: string) {
  if (driver === "mysql") {
    return `\`${value.split("`").join("``")}\``;
  }
  return `"${value.split('"').join('""')}"`;
}

function qualifiedSqlName(driver: string, table: DatabaseTable) {
  const name = quoteDbIdentifier(driver, table.name);
  // PostgreSQL qualifies by schema; MySQL qualifies by its database (catalog).
  const qualifier = table.schema ?? table.catalog;
  return qualifier ? `${quoteDbIdentifier(driver, qualifier)}.${name}` : name;
}

function generateSelectSql(driver: string, table: DatabaseTable) {
  const columns = table.columns.length
    ? table.columns.map((column) => quoteDbIdentifier(driver, column.name)).join(", ")
    : "*";
  return `SELECT ${columns}\nFROM ${qualifiedSqlName(driver, table)}\nLIMIT 100;`;
}

function generateInsertSql(driver: string, table: DatabaseTable) {
  if (!table.columns.length) {
    return `INSERT INTO ${qualifiedSqlName(driver, table)} () VALUES ();`;
  }
  const columns = table.columns.map((column) => quoteDbIdentifier(driver, column.name)).join(", ");
  const placeholders = table.columns.map(() => "NULL").join(", ");
  return `INSERT INTO ${qualifiedSqlName(driver, table)} (${columns})\nVALUES (${placeholders});`;
}

function ConnectionActions({
  connection,
  onConnect,
  onDeleteConnection,
  onDisconnect,
  onEditConnection,
  onNewQuery,
  onRefresh,
  onRefreshSchema,
  status,
}: {
  connection: DatabaseConnection;
  onConnect?: (connection: DatabaseConnection) => void;
  onDeleteConnection?: (connection: DatabaseConnection) => void;
  onDisconnect?: (connection: DatabaseConnection) => void;
  onEditConnection?: (connection: DatabaseConnection) => void;
  onNewQuery?: () => void;
  onRefresh?: () => void;
  onRefreshSchema?: (connection: DatabaseConnection) => void;
  status: DatabaseConnectionStatus;
}) {
  const { t } = useI18n();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <IconButton label={t("database.tree.actionsLabel", { name: connection.name })} size="compact">
          <MoreHorizontal size={13} />
        </IconButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuItem onSelect={() => onConnect?.(connection)}>{t("common.actions.connect")}</DropdownMenuItem>
        <DropdownMenuItem disabled={status === "disconnected"} onSelect={() => onDisconnect?.(connection)}>
          {t("common.actions.disconnect")}
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={onNewQuery}>{t("database.actions.newQuery")}</DropdownMenuItem>
        <DropdownMenuItem onSelect={onRefresh}>{t("database.actions.refreshConnections")}</DropdownMenuItem>
        <DropdownMenuItem onSelect={() => onRefreshSchema?.(connection)}>{t("database.actions.refreshSchema")}</DropdownMenuItem>
        {onEditConnection && (
          <DropdownMenuItem onSelect={() => onEditConnection(connection)}>
            <Pencil size={13} />
            {t("database.tree.editConnection")}
          </DropdownMenuItem>
        )}
        <DropdownMenuItem onSelect={() => void navigator.clipboard?.writeText(connection.name)}>
          <Copy size={13} />
          {t("database.tree.copyName")}
        </DropdownMenuItem>
        {onDeleteConnection && (
          <DropdownMenuItem
            className="text-[var(--u-color-danger)]"
            onSelect={() => onDeleteConnection(connection)}
          >
            <Trash2 size={13} />
            {t("database.tree.deleteConnection")}
          </DropdownMenuItem>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function ConnectionContextMenu({
  connection,
  onConnect,
  onDeleteConnection,
  onDisconnect,
  onEditConnection,
  onNewQuery,
  onRefreshSchema,
  status,
}: {
  connection: DatabaseConnection;
  onConnect?: (connection: DatabaseConnection) => void;
  onDeleteConnection?: (connection: DatabaseConnection) => void;
  onDisconnect?: (connection: DatabaseConnection) => void;
  onEditConnection?: (connection: DatabaseConnection) => void;
  onNewQuery?: () => void;
  onRefreshSchema?: (connection: DatabaseConnection) => void;
  status: DatabaseConnectionStatus;
}) {
  const { t } = useI18n();

  return (
    <>
      <ContextMenuItem onSelect={() => onConnect?.(connection)}>
        {t("common.actions.connect")}
      </ContextMenuItem>
      <ContextMenuItem
        disabled={status === "disconnected"}
        onSelect={() => onDisconnect?.(connection)}
      >
        {t("common.actions.disconnect")}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onNewQuery}>{t("database.actions.newQuery")}</ContextMenuItem>
      <ContextMenuItem onSelect={() => onRefreshSchema?.(connection)}>
        {t("database.actions.refreshSchema")}
      </ContextMenuItem>
      {onEditConnection && (
        <ContextMenuItem onSelect={() => onEditConnection(connection)}>
          {t("database.tree.editConnection")}
        </ContextMenuItem>
      )}
      <ContextMenuItem onSelect={() => void navigator.clipboard?.writeText(connection.name)}>
        {t("database.tree.copyName")}
      </ContextMenuItem>
      {onDeleteConnection && (
        <ContextMenuItem onSelect={() => onDeleteConnection(connection)} tone="danger">
          {t("database.tree.deleteConnection")}
        </ContextMenuItem>
      )}
    </>
  );
}

function resolveConnectionStatus({
  hasSchema,
  selected,
  session,
}: {
  hasSchema: boolean;
  selected: boolean;
  session?: DatabaseConnectionSessionState;
}): DatabaseConnectionStatus {
  if (session?.status) {
    return session.status;
  }
  if (selected && hasSchema) {
    return "connected";
  }
  return "disconnected";
}

function databaseLabel(connection: DatabaseConnection) {
  if (connection.driver === "sqlite") {
    return connection.sqlitePath?.split(/[\\/]/).pop() || connection.name;
  }
  return connection.database || connection.name;
}

function connectionStateTitle(
  connection: DatabaseConnection,
  session?: DatabaseConnectionSessionState,
) {
  const message = session?.message ? ` - ${session.message}` : "";
  return `${connection.name} (${connection.driver})${message}`;
}

export function DatabaseSidebarToolbar({
  onNewQuery,
  onRefresh,
}: {
  onNewQuery?: () => void;
  onRefresh?: () => void;
}) {
  const { t } = useI18n();

  return (
    <div className="flex items-center gap-1">
      <IconButton label={t("database.actions.newQueryLabel")} onClick={onNewQuery}>
        <Play size={13} />
      </IconButton>
      <IconButton label={t("database.connection.refreshLabel")} onClick={onRefresh}>
        <RefreshCw size={13} />
      </IconButton>
    </div>
  );
}
