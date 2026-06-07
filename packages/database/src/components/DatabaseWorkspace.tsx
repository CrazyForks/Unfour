import type { DatabaseConnection, DatabaseQueryResult } from "@unfour/command-client";
import type { DatabaseTableViewState } from "../model/types";
import { Tabs } from "@unfour/ui";
import { QueryResultPanel } from "./QueryResultPanel";
import { SqlEditorTab } from "./SqlEditorTab";
import { TableDataTab } from "./TableDataTab";

export function DatabaseWorkspace({
  activeResultTab,
  activeTabId,
  connections,
  error,
  executePending,
  onRun,
  onSelectConnection,
  onSelectResultTab,
  onSelectTab,
  onSqlChange,
  onStop,
  pendingConfirmation,
  queryResult,
  selectedConnectionId,
  sql,
  tableView,
}: {
  activeResultTab: "results" | "messages" | "logs";
  activeTabId: string;
  connections: DatabaseConnection[];
  error: unknown;
  executePending: boolean;
  onRun: () => void;
  onSelectConnection: (connectionId: string) => void;
  onSelectResultTab: (tab: "results" | "messages" | "logs") => void;
  onSelectTab: (tabId: string) => void;
  onSqlChange: (sql: string) => void;
  onStop: () => void;
  pendingConfirmation: boolean;
  queryResult: DatabaseQueryResult | null;
  selectedConnectionId: string | null;
  sql: string;
  tableView: DatabaseTableViewState | null;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <Tabs
        activeId={activeTabId}
        onSelect={onSelectTab}
        tabs={[
          {
            id: "sql-editor",
            loading: executePending,
            modified: sql.trim().length > 0,
            title: "SQL Editor",
          },
          {
            id: "table-data",
            loading: executePending,
            meta: tableView ? <span className="text-[11px] text-[var(--u-color-text-soft)]">{tableView.tableName}</span> : null,
            title: "Table Data",
          },
          { id: "table-structure", title: "Table Structure" },
        ]}
      />
      <div className="flex min-h-0 flex-1 flex-col">
        {activeTabId === "table-data" ? (
          <TableDataTab result={queryResult} />
        ) : (
          <SqlEditorTab
            connections={connections}
            executePending={executePending}
            onRun={onRun}
            onSelectConnection={onSelectConnection}
            onSqlChange={onSqlChange}
            onStop={onStop}
            pendingConfirmation={pendingConfirmation}
            selectedConnectionId={selectedConnectionId}
            sql={sql}
          />
        )}
        <QueryResultPanel
          activeTab={activeResultTab}
          error={error}
          isPending={executePending}
          onSelectTab={onSelectResultTab}
          pendingConfirmation={pendingConfirmation}
          result={queryResult}
        />
      </div>
    </div>
  );
}
