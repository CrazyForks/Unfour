import { CheckCircle2, Database, Plus, Save, Table2, Trash2, XCircle } from "lucide-react";
import { FormEvent, type ReactNode, useEffect, useMemo, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  deleteDatabaseConnection,
  saveDatabaseConnection,
  testDatabaseConnection,
} from "@unfour/command-client";
import type {
  DatabaseConnectionInput,
  DatabaseQueryResult,
  DatabaseTable,
  DatabaseTestResult,
} from "@unfour/command-client";
import { useWorkspaceStore } from "@unfour/workspace";
import {
  Badge,
  Button,
  ErrorState,
  IconButton,
  Input,
  Select,
  StatusBadge,
  Toolbar,
  ToolbarGroup,
} from "@unfour/ui";
import { DatabaseModuleToolbar } from "./components/DatabaseModuleToolbar";
import { DatabaseStatusBar } from "./components/DatabaseStatusBar";
import { DatabaseWorkspace } from "./components/DatabaseWorkspace";
import { SchemaTree } from "./components/SchemaTree";
import { TableInspector } from "./components/TableInspector";
import { useDatabaseConnections } from "./hooks/useDatabaseConnections";
import { useDatabaseLayout } from "./hooks/useDatabaseLayout";
import { useSchemaTree } from "./hooks/useSchemaTree";
import { useSqlExecution } from "./hooks/useSqlExecution";
import { useTableData } from "./hooks/useTableData";
import { defaultSql } from "./model/database-state";
import type { DatabaseTableViewState } from "./model/types";
import { formatDatabaseError } from "./result-utils";

export function DatabasePage({ workspaceId }: { workspaceId: string }) {
  const queryClient = useQueryClient();
  const {
    selectedDatabaseConnectionId: selectedConnectionId,
    setSelectedDatabaseConnection,
  } = useWorkspaceStore();
  const layout = useDatabaseLayout();
  const [testResult, setTestResult] = useState<DatabaseTestResult | null>(null);
  const [queryResult, setQueryResult] = useState<DatabaseQueryResult | null>(null);
  const [pendingSqlConfirmation, setPendingSqlConfirmation] = useState(false);
  const [sql, setSql] = useState(defaultSql);
  const [tableView, setTableView] = useState<DatabaseTableViewState | null>(null);
  const [selectedTable, setSelectedTable] = useState<DatabaseTable | null>(null);
  const [form, setForm] = useState<DatabaseConnectionInput>({
    workspaceId,
    name: "Local SQLite",
    driver: "sqlite",
    sqlitePath: "",
  });

  const connectionsQuery = useDatabaseConnections(workspaceId);
  const connections = connectionsQuery.data ?? [];
  const selectedConnection = useMemo(
    () => connections.find((item) => item.id === selectedConnectionId) ?? null,
    [connections, selectedConnectionId],
  );
  const schemaQuery = useSchemaTree({
    connection: selectedConnection,
    connectionId: selectedConnectionId,
    workspaceId,
  });

  useEffect(() => {
    if (!connections.length) {
      setSelectedDatabaseConnection(null);
      return;
    }

    if (!selectedConnectionId || !connections.some((connection) => connection.id === selectedConnectionId)) {
      setSelectedDatabaseConnection(connections[0].id);
    }
  }, [connections, selectedConnectionId, setSelectedDatabaseConnection]);

  useEffect(() => {
    setForm((current) => ({ ...current, workspaceId }));
  }, [workspaceId]);

  useEffect(() => {
    if (!selectedConnection) {
      return;
    }

    setForm({
      id: selectedConnection.id,
      workspaceId,
      name: selectedConnection.name,
      driver: selectedConnection.driver,
      host: selectedConnection.host,
      port: selectedConnection.port,
      database: selectedConnection.database,
      username: selectedConnection.username,
      sqlitePath: selectedConnection.sqlitePath,
      credentialRef: selectedConnection.credentialRef,
    });
    setTestResult(null);
    setQueryResult(null);
    setTableView(null);
    setSelectedTable(null);
    setPendingSqlConfirmation(false);
  }, [selectedConnection, workspaceId]);

  const saveMutation = useMutation({
    mutationFn: saveDatabaseConnection,
    onSuccess: (connection) => {
      setSelectedDatabaseConnection(connection.id);
      queryClient.invalidateQueries({ queryKey: ["database-connections", workspaceId] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (connectionId: string) => deleteDatabaseConnection(workspaceId, connectionId),
    onSuccess: () => {
      setSelectedDatabaseConnection(null);
      setTestResult(null);
      setQueryResult(null);
      setTableView(null);
      setSelectedTable(null);
      setPendingSqlConfirmation(false);
      queryClient.invalidateQueries({ queryKey: ["database-connections", workspaceId] });
    },
  });

  const testMutation = useMutation({
    mutationFn: (connectionId: string) => testDatabaseConnection(workspaceId, connectionId),
    onSuccess: (result) => {
      setTestResult(result);
      queryClient.invalidateQueries({ queryKey: ["database-schema", workspaceId, selectedConnectionId] });
    },
  });

  const executeMutation = useSqlExecution({
    connectionId: selectedConnectionId,
    onConfirmationRequired: setPendingSqlConfirmation,
    onExecuteStart: () => {
      setTableView(null);
      layout.setActiveTabId("sql-editor");
    },
    onSuccess: (result) => {
      setTableView(null);
      setQueryResult(result);
    },
    sql,
    workspaceId,
  });

  const browseMutation = useTableData({
    connectionId: selectedConnectionId,
    onBrowseStart: () => {
      setPendingSqlConfirmation(false);
      layout.setActiveTabId("table-data");
    },
    onSuccess: (browse) => {
      setPendingSqlConfirmation(false);
      setSql(browse.sql);
      setQueryResult(browse.result);
      setTableView({
        pageIndex: Math.floor(browse.offset / Math.max(1, browse.limit)),
        pageSize: browse.limit,
        readOnly: browse.readOnly,
        tableName: browse.tableName,
        totalRows: browse.totalRows,
      });
    },
    workspaceId,
  });

  function updateForm(patch: Partial<DatabaseConnectionInput>) {
    setForm((current) => ({ ...current, ...patch, workspaceId }));
  }

  function submitConnection(event: FormEvent) {
    event.preventDefault();
    saveMutation.mutate({
      ...form,
      credentialRef: form.credentialRef?.trim() || null,
      sqlitePath: form.sqlitePath?.trim() || null,
      host: form.host?.trim() || null,
      database: form.database?.trim() || null,
      username: form.username?.trim() || null,
    });
  }

  function newConnection() {
    setSelectedDatabaseConnection(null);
    setTestResult(null);
    setQueryResult(null);
    setTableView(null);
    setSelectedTable(null);
    setPendingSqlConfirmation(false);
    setForm({ workspaceId, name: "Local SQLite", driver: "sqlite", sqlitePath: "" });
  }

  function browseTablePage(table: DatabaseTable, pageIndex: number, pageSize: number) {
    setSelectedTable(table);
    browseMutation.mutate({
      pageIndex: Math.max(0, pageIndex),
      pageSize,
      schema: table.schema,
      tableName: table.name,
    });
  }

  const activeError = layout.activeTabId === "table-data" ? browseMutation.error : executeMutation.error;

  return (
    <div className="flex h-full min-h-0 min-w-0 flex-col bg-[var(--u-color-surface)]">
      <DatabaseModuleToolbar
        connections={connections}
        executePending={executeMutation.isPending || browseMutation.isPending}
        onNewQuery={() => {
          layout.setActiveTabId("sql-editor");
          setTableView(null);
        }}
        onRefresh={() => {
          queryClient.invalidateQueries({ queryKey: ["database-connections", workspaceId] });
          queryClient.invalidateQueries({ queryKey: ["database-schema", workspaceId, selectedConnectionId] });
        }}
        onRun={() => executeMutation.mutate(pendingSqlConfirmation)}
        onSelectConnection={setSelectedDatabaseConnection}
        onStop={() => undefined}
        pendingConfirmation={pendingSqlConfirmation}
        selectedConnectionId={selectedConnectionId}
      />
      <div className="grid min-h-0 flex-1 grid-cols-[280px_minmax(0,1fr)_minmax(240px,300px)]">
        <aside className="min-h-0 border-r border-[var(--u-color-border)] bg-[var(--u-color-surface-subtle)]">
          <ConnectionEditor
            error={saveMutation.error ?? testMutation.error}
            form={form}
            onDelete={() => selectedConnectionId && deleteMutation.mutate(selectedConnectionId)}
            onNew={newConnection}
            onSubmit={submitConnection}
            onTest={() => selectedConnectionId && testMutation.mutate(selectedConnectionId)}
            onUpdate={updateForm}
            result={testResult}
            savePending={saveMutation.isPending}
            selectedConnectionId={selectedConnectionId}
            testPending={testMutation.isPending}
          />
          <div className="min-h-0 overflow-auto border-t border-[var(--u-color-border)] p-2">
            <Toolbar className="mb-2 h-8 border border-[var(--u-color-border)]">
              <ToolbarGroup>
                <Table2 size={14} />
                <span className="text-[12px] font-semibold text-[var(--u-color-text)]">Schema</span>
              </ToolbarGroup>
              <ToolbarGroup>
                <Badge tone="neutral">{schemaQuery.data?.tables.length ?? 0}</Badge>
              </ToolbarGroup>
            </Toolbar>
            <SchemaTree
              disabled={!selectedConnectionId || browseMutation.isPending}
              error={schemaQuery.error}
              loading={schemaQuery.isFetching}
              onBrowse={(table) => browseTablePage(table, 0, tableView?.pageSize ?? 100)}
              schema={schemaQuery.data}
            />
          </div>
        </aside>
        <DatabaseWorkspace
          activeResultTab={layout.resultTab}
          activeTabId={layout.activeTabId}
          connections={connections}
          error={activeError}
          executePending={executeMutation.isPending || browseMutation.isPending}
          onRun={() => executeMutation.mutate(pendingSqlConfirmation)}
          onSelectConnection={setSelectedDatabaseConnection}
          onSelectResultTab={layout.setResultTab}
          onSelectTab={layout.setActiveTabId}
          onSqlChange={setSql}
          onStop={() => undefined}
          pendingConfirmation={pendingSqlConfirmation}
          queryResult={queryResult}
          selectedConnectionId={selectedConnectionId}
          sql={sql}
          tableView={tableView}
        />
        <aside className="min-h-0 border-l border-[var(--u-color-border)] bg-[var(--u-color-surface-subtle)]">
          <TableInspector
            activeTab={layout.inspectorTab}
            onSelectTab={layout.setInspectorTab}
            table={selectedTable}
          />
        </aside>
      </div>
      <DatabaseStatusBar connection={selectedConnection} executing={executeMutation.isPending || browseMutation.isPending} />
    </div>
  );
}

function ConnectionEditor({
  error,
  form,
  onDelete,
  onNew,
  onSubmit,
  onTest,
  onUpdate,
  result,
  savePending,
  selectedConnectionId,
  testPending,
}: {
  error: unknown;
  form: DatabaseConnectionInput;
  onDelete: () => void;
  onNew: () => void;
  onSubmit: (event: FormEvent) => void;
  onTest: () => void;
  onUpdate: (patch: Partial<DatabaseConnectionInput>) => void;
  result: DatabaseTestResult | null;
  savePending: boolean;
  selectedConnectionId: string | null;
  testPending: boolean;
}) {
  return (
    <form className="space-y-2 p-2" onSubmit={onSubmit}>
      <Toolbar className="h-8 border border-[var(--u-color-border)]">
        <ToolbarGroup>
          <Database size={14} />
          <span className="text-[12px] font-semibold text-[var(--u-color-text)]">Connections</span>
        </ToolbarGroup>
        <ToolbarGroup>
          <IconButton label="New database connection" onClick={onNew}>
            <Plus size={13} />
          </IconButton>
        </ToolbarGroup>
      </Toolbar>
      <Field title="Name">
        <Input onChange={(event) => onUpdate({ name: event.target.value })} value={form.name} />
      </Field>
      <Field title="Driver">
        <Select
          onChange={(event) =>
            onUpdate({
              driver: event.target.value as DatabaseConnectionInput["driver"],
              sqlitePath: event.target.value === "sqlite" ? form.sqlitePath : null,
              credentialRef: event.target.value === "sqlite" ? null : form.credentialRef,
            })
          }
          options={[
            { label: "SQLite", value: "sqlite" },
            { label: "PostgreSQL", value: "postgres" },
            { label: "MySQL / MariaDB", value: "mysql" },
          ]}
          value={form.driver}
        />
      </Field>
      {form.driver === "sqlite" ? (
        <Field title="SQLite Path">
          <Input onChange={(event) => onUpdate({ sqlitePath: event.target.value })} placeholder="E:\\data\\app.sqlite" value={form.sqlitePath ?? ""} />
        </Field>
      ) : (
        <>
          <div className="grid grid-cols-[1fr_76px] gap-2">
            <Field title="Host">
              <Input onChange={(event) => onUpdate({ host: event.target.value })} placeholder="127.0.0.1" value={form.host ?? ""} />
            </Field>
            <Field title="Port">
              <Input
                onChange={(event) => onUpdate({ port: event.target.value ? Number(event.target.value) : null })}
                placeholder={form.driver === "postgres" ? "5432" : "3306"}
                type="number"
                value={form.port ?? ""}
              />
            </Field>
          </div>
          <Field title="Database">
            <Input onChange={(event) => onUpdate({ database: event.target.value })} value={form.database ?? ""} />
          </Field>
          <Field title="Username">
            <Input onChange={(event) => onUpdate({ username: event.target.value })} value={form.username ?? ""} />
          </Field>
          <Field title="Credential Ref">
            <Input onChange={(event) => onUpdate({ credentialRef: event.target.value })} value={form.credentialRef ?? ""} />
          </Field>
        </>
      )}
      <div className="flex items-center gap-1">
        <Button disabled={savePending} size="sm" type="submit">
          <Save size={13} />
          Save
        </Button>
        <Button disabled={!selectedConnectionId || testPending} onClick={onTest} size="sm" type="button" variant="outline">
          <CheckCircle2 size={13} />
          Test
        </Button>
        <IconButton disabled={!selectedConnectionId} label="Delete database connection" onClick={onDelete}>
          <Trash2 size={13} />
        </IconButton>
      </div>
      {error ? <ErrorState className="min-h-[48px]">{formatDatabaseError(error)}</ErrorState> : null}
      {result && (
        <div className="flex items-center gap-2 text-[12px] text-[var(--u-color-text-muted)]">
          {result.ok ? <CheckCircle2 size={13} /> : <XCircle size={13} />}
          <span className="min-w-0 flex-1 truncate">{String(result.message)}</span>
          <StatusBadge tone={result.ok ? "success" : "warning"}>{result.ok ? "ok" : "failed"}</StatusBadge>
        </div>
      )}
    </form>
  );
}

function Field({
  children,
  title,
}: {
  children: ReactNode;
  title: string;
}) {
  return (
    <label className="block space-y-1">
      <span className="text-[11px] font-medium uppercase text-[var(--u-color-text-soft)]">{title}</span>
      {children}
    </label>
  );
}
