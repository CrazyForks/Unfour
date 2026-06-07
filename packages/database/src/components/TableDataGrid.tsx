import type { DatabaseQueryResult } from "@unfour/command-client";
import { DataTable, type DataTableColumn, StatusBadge } from "@unfour/ui";

export function TableDataGrid({ result }: { result: DatabaseQueryResult }) {
  const columns: DataTableColumn<unknown[]>[] = result.columns.map((column, columnIndex) => ({
    cell: (row) => renderCell(row[columnIndex]),
    header: column.name,
    id: column.name || `column-${columnIndex}`,
    meta: column.dataType,
    width: Math.min(Math.max(column.name.length * 9 + 96, 140), 360),
  }));

  return (
    <DataTable
      className="flex-1"
      columns={columns}
      empty="This result set is empty."
      getRowKey={(_, index) => index}
      rows={result.rows}
    />
  );
}

function renderCell(value: unknown) {
  if (value === null || value === undefined) {
    return <StatusBadge>NULL</StatusBadge>;
  }

  if (typeof value === "boolean") {
    return <StatusBadge tone={value ? "success" : "neutral"}>{String(value)}</StatusBadge>;
  }

  if (value instanceof Uint8Array) {
    return <StatusBadge>binary</StatusBadge>;
  }

  if (typeof value === "object") {
    return <span className="font-mono text-[12px]">{JSON.stringify(value)}</span>;
  }

  return String(value);
}
