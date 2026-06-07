import { Columns3, Table2 } from "lucide-react";
import type { DatabaseSchema, DatabaseTable } from "@unfour/command-client";
import { Badge, EmptyState, ErrorState, IconButton, LoadingState, TreeView, type TreeViewItem } from "@unfour/ui";
import { formatDatabaseError } from "../result-utils";

export function SchemaTree({
  disabled,
  error,
  loading,
  onBrowse,
  schema,
}: {
  disabled: boolean;
  error: unknown;
  loading: boolean;
  onBrowse: (table: DatabaseTable) => void;
  schema?: DatabaseSchema;
}) {
  if (error) {
    return <ErrorState className="min-h-[72px]">{formatDatabaseError(error)}</ErrorState>;
  }

  if (loading) {
    return <LoadingState className="min-h-[72px]">Loading schema...</LoadingState>;
  }

  if (!schema?.tables.length) {
    return <EmptyState className="min-h-[72px]">Select a SQLite connection to inspect tables.</EmptyState>;
  }

  const items: TreeViewItem[] = schema.tables.map((table) => ({
    actions: (
      <IconButton disabled={disabled} label={`View table data for ${table.name}`} onClick={() => onBrowse(table)}>
        <Table2 size={13} />
      </IconButton>
    ),
    children: table.columns.map((column) => ({
      icon: <Columns3 size={12} />,
      id: `${table.name}:${column.name}`,
      label: column.name,
      meta: (
        <span className="text-[10px] uppercase text-[var(--u-color-text-soft)]">
          {column.dataType || "ANY"}
        </span>
      ),
      title: column.name,
    })),
    icon: <Table2 size={13} />,
    id: table.name,
    label: table.name,
    meta: <Badge tone="neutral">{table.kind}</Badge>,
    title: table.name,
  }));

  return <TreeView defaultExpandedIds={schema.tables.slice(0, 2).map((table) => table.name)} items={items} />;
}
