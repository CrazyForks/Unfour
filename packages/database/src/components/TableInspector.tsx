import type { DatabaseTable } from "@unfour/command-client";
import { EmptyState, Tabs } from "@unfour/ui";

export function TableInspector({
  activeTab,
  onSelectTab,
  table,
}: {
  activeTab: "columns" | "indexes" | "constraints" | "properties" | "ddl";
  onSelectTab: (tab: "columns" | "indexes" | "constraints" | "properties" | "ddl") => void;
  table: DatabaseTable | null;
}) {
  return (
    <div className="flex h-full min-h-0 flex-col">
      <Tabs
        activeId={activeTab}
        className="h-[30px] px-1"
        onSelect={(tabId) => onSelectTab(tabId as "columns" | "indexes" | "constraints" | "properties" | "ddl")}
        tabs={[
          { id: "columns", title: "Columns" },
          { id: "indexes", title: "Indexes" },
          { id: "constraints", title: "Constraints" },
          { id: "properties", title: "Properties" },
          { id: "ddl", title: "DDL" },
        ]}
      />
      {!table ? (
        <EmptyState className="m-2 min-h-0 flex-1">Select a database object to inspect details.</EmptyState>
      ) : activeTab === "columns" ? (
        <div className="min-h-0 flex-1 overflow-auto p-2 text-[12px]">
          {table.columns.map((column) => (
            <div
              className="grid min-h-[var(--u-size-table-row)] grid-cols-[minmax(0,1fr)_88px_36px] items-center gap-2 border-b border-[var(--u-color-border)]"
              key={column.name}
            >
              <span className="truncate text-[var(--u-color-text)]">{column.name}</span>
              <span className="truncate text-[var(--u-color-text-muted)]">{column.dataType || "ANY"}</span>
              <span className="text-[var(--u-color-text-soft)]">{column.primaryKey ? "pk" : ""}</span>
            </div>
          ))}
        </div>
      ) : (
        <EmptyState className="m-2 min-h-0 flex-1">This metadata is unavailable from the current backend.</EmptyState>
      )}
    </div>
  );
}
