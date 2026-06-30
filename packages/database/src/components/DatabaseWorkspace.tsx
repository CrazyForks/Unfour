import type {
  DatabaseConnection,
  DatabaseQueryResult,
  DatabaseSchema,
  DatabaseTable,
  DatabaseTableStructure,
} from "@unfour/command-client";
import type {
  DatabaseResultTab,
  DatabaseTableViewState,
  DatabaseWorkspaceTabId,
  SqlHistoryEntry,
  TableEditing,
  TableSegment,
} from "../model/types";
import { SplitPane, useI18n, type WorkspaceTab } from "@unfour/ui";
import { QueryResultPanel } from "./QueryResultPanel";
import { SqlEditorTab } from "./SqlEditorTab";
import { TableDataTab } from "./TableDataTab";
import { TableInspector } from "./TableInspector";

type StructureTab = "ddl" | "indexes" | "constraints" | "properties";

export function DatabaseWorkspace({
  activeResultTab,
  activeStructureTab,
  activeTabId,
  catalogOptions,
  connections,
  error,
  executePending,
  history,
  onChangeQueryContext,
  onClearHistory,
  onClearSql,
  onPreviewSelectedTable,
  onRefreshSchema,
  onRun,
  onSelectConnection,
  queryCatalog,
  querySchema,
  schemaOptions,
  onSelectHistory,
  onSelectStructureTab,
  onSelectResultTab,
  onSelectTab,
  onSelectTableSegment,
  onShowHistory,
  onSqlChange,
  onStop,
  onTableFilter,
  onTablePageChange,
  onTableSort,
  pendingConfirmation,
  queryResult,
  schema,
  schemaError,
  selectedConnectionId,
  selectedTable,
  sql,
  structure,
  structureError,
  structureLoading,
  tableEditing,
  tableFilter,
  tableSegment,
  tableSort,
  tableView,
  workspaceId,
}: {
  activeResultTab: DatabaseResultTab;
  activeStructureTab: StructureTab;
  activeTabId: DatabaseWorkspaceTabId;
  catalogOptions: string[];
  connections: DatabaseConnection[];
  error: unknown;
  executePending: boolean;
  history: SqlHistoryEntry[];
  onChangeQueryContext: (patch: { catalog?: string | null; schema?: string | null }) => void;
  onClearHistory: () => void;
  onClearSql: () => void;
  onPreviewSelectedTable: () => void;
  onRefreshSchema: () => void;
  onRun: (selectedSql?: string) => void;
  onSelectConnection: (connectionId: string) => void;
  queryCatalog: string | null;
  querySchema: string | null;
  schemaOptions: string[];
  onSelectHistory: (entry: SqlHistoryEntry) => void;
  onSelectStructureTab: (tab: StructureTab) => void;
  onSelectResultTab: (tab: DatabaseResultTab) => void;
  onSelectTab: (tabId: DatabaseWorkspaceTabId) => void;
  onSelectTableSegment: (segment: TableSegment) => void;
  onShowHistory: () => void;
  onSqlChange: (sql: string) => void;
  onStop: () => void;
  onTableFilter: (filter: string) => void;
  onTablePageChange: (pageIndex: number, pageSize: number) => void;
  onTableSort: (column: string) => void;
  pendingConfirmation: boolean;
  queryResult: DatabaseQueryResult | null;
  schema?: DatabaseSchema;
  schemaError: unknown;
  selectedConnectionId: string | null;
  selectedTable: DatabaseTable | null;
  sql: string;
  structure?: DatabaseTableStructure | null;
  structureError?: unknown;
  structureLoading?: boolean;
  tableEditing?: TableEditing | null;
  tableFilter: string;
  tableSegment: TableSegment;
  tableSort: { column: string; descending: boolean } | null;
  tableView: DatabaseTableViewState | null;
  workspaceId: string;
}) {
  const { t } = useI18n();
  const isTableTab = activeTabId === "table";

  const tabs: WorkspaceTab[] = [
    {
      id: "table",
      loading: isTableTab && executePending,
      meta: selectedTable ? (
        <span className="text-[11px] text-[var(--u-color-text-soft)]">{selectedTable.name}</span>
      ) : null,
      title: t("database.editor.tableTab"),
    },
    {
      id: "query",
      loading: !isTableTab && executePending,
      modified: sql.trim().length > 0,
      title: t("database.editor.queryConsole"),
    },
  ];

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {/* Merged Tab bar + SegmentedControl row: eliminates a separate SegmentedControl bar. */}
      <div
        className="flex h-[var(--u-size-tabbar)] shrink-0 items-end justify-between overflow-x-auto border-b border-[var(--u-color-border)] bg-[var(--u-color-surface-subtle)] px-2"
        role="tablist"
      >
        <div className="flex items-end">
          {tabs.map((tab) => {
            const active = tab.id === activeTabId;
            return (
              <div
                className={
                  "group flex h-[30px] min-w-[120px] max-w-[220px] items-center gap-2 rounded-t-[var(--u-radius-sm)] border px-2 text-[12px] font-medium transition-colors " +
                  (active
                    ? "border-[var(--u-color-border)] border-b-[var(--u-color-surface)] bg-[var(--u-color-surface)] text-[var(--u-color-text)]"
                    : "border-transparent text-[var(--u-color-text-muted)] hover:bg-[var(--u-color-surface-hover)] hover:text-[var(--u-color-text)]")
                }
                key={tab.id}
              >
                <button
                  aria-selected={active}
                  className="flex min-w-0 flex-1 items-center gap-2 focus-visible:outline-none"
                  onClick={() => onSelectTab(tab.id as DatabaseWorkspaceTabId)}
                  role="tab"
                  type="button"
                >
                  <span className="min-w-0 flex-1 truncate text-left">
                    {tab.modified ? "* " : ""}
                    {tab.title}
                  </span>
                  {tab.loading && (
                    <span className="h-1.5 w-1.5 rounded-full bg-[var(--u-color-primary)]" />
                  )}
                  {tab.meta}
                </button>
              </div>
            );
          })}
        </div>
        {isTableTab && (
          <div className="ml-3 shrink-0 pb-0.5">
            <SegmentedControl
              onChange={onSelectTableSegment}
              options={[
                { label: t("database.editor.dataView"), value: "data" },
                { label: t("database.editor.structureView"), value: "structure" },
              ]}
              value={tableSegment}
            />
          </div>
        )}
      </div>
      <div className="flex min-h-0 flex-1 flex-col">
        {isTableTab ? (
          <>
            {tableSegment === "data" ? (
              <TableDataTab
                editing={tableEditing}
                error={error}
                executePending={executePending}
                onPageChange={onTablePageChange}
                onRefresh={() => tableView && onTablePageChange(tableView.pageIndex, tableView.pageSize)}
                onTableFilter={onTableFilter}
                onTableSort={onTableSort}
                result={queryResult}
                tableFilter={tableFilter}
                tableSort={tableSort}
                tableView={tableView}
              />
            ) : (
              <TableInspector
                activeTab={activeStructureTab}
                error={structureError ?? schemaError}
                loading={Boolean(structureLoading)}
                onPreview={onPreviewSelectedTable}
                onRefresh={onRefreshSchema}
                onSelectTab={onSelectStructureTab}
                previewPending={executePending}
                structure={structure}
                table={selectedTable}
              />
            )}
          </>
        ) : (
          <SplitPane className="min-h-0 flex-1" defaultRatio={62} minPaneSize={220} orientation="vertical" resizable>
            <SqlEditorTab
              catalogOptions={catalogOptions}
              connections={connections}
              executePending={executePending}
              onChangeQueryContext={onChangeQueryContext}
              onClearSql={onClearSql}
              onRun={onRun}
              onSelectConnection={onSelectConnection}
              onShowHistory={onShowHistory}
              onSqlChange={onSqlChange}
              onStop={onStop}
              pendingConfirmation={pendingConfirmation}
              queryCatalog={queryCatalog}
              querySchema={querySchema}
              schema={schema}
              schemaOptions={schemaOptions}
              selectedConnectionId={selectedConnectionId}
              sql={sql}
              workspaceId={workspaceId}
            />
            <QueryResultPanel
              activeTab={activeResultTab}
              error={error}
              history={history}
              isPending={executePending}
              onClearHistory={onClearHistory}
              onSelectHistory={onSelectHistory}
              onSelectTab={onSelectResultTab}
              pendingConfirmation={pendingConfirmation}
              result={queryResult}
            />
          </SplitPane>
        )}
      </div>
    </div>
  );
}

function SegmentedControl<T extends string>({
  onChange,
  options,
  value,
}: {
  onChange: (value: T) => void;
  options: Array<{ label: string; value: T }>;
  value: T;
}) {
  return (
    <div className="inline-flex items-center gap-0.5 rounded-[7px] border border-[var(--u-color-border)] bg-[var(--u-color-surface-subtle)] p-0.5">
      {options.map((option) => {
        const active = option.value === value;
        return (
          <button
            className={`inline-flex h-[22px] items-center rounded-[5px] px-3 text-[12px] font-semibold ${
              active
                ? "bg-[var(--u-color-surface)] text-[var(--u-color-text)] shadow-[0_1px_2px_rgba(0,0,0,0.08)]"
                : "text-[var(--u-color-text-muted)] hover:text-[var(--u-color-text)]"
            }`}
            key={option.value}
            onClick={() => onChange(option.value)}
            type="button"
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
