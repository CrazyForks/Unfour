import * as React from "react";
import { cn } from "./utils";

export type DataTableColumn<T> = {
  align?: "left" | "right";
  cell?: (row: T, rowIndex: number) => React.ReactNode;
  id: string;
  header: React.ReactNode;
  meta?: React.ReactNode;
  width?: number;
};

export function DataTable<T>({
  className,
  columns,
  empty,
  getRowKey,
  rows,
}: {
  className?: string;
  columns: DataTableColumn<T>[];
  empty?: React.ReactNode;
  getRowKey?: (row: T, rowIndex: number) => React.Key;
  rows: T[];
}) {
  return (
    <div className={cn("min-h-0 overflow-auto", className)}>
      <table className="w-max min-w-full table-fixed text-left text-[12px]">
        <colgroup>
          {columns.map((column) => (
            <col key={column.id} style={column.width ? { width: column.width } : undefined} />
          ))}
        </colgroup>
        <thead className="sticky top-0 z-10 bg-[var(--u-color-surface-subtle)] text-[var(--u-color-text-muted)]">
          <tr>
            {columns.map((column) => (
              <th
                className={cn(
                  "h-[var(--u-size-table-row)] border-b border-[var(--u-color-border)] px-2 font-medium",
                  column.align === "right" && "text-right",
                )}
                key={column.id}
              >
                <div className="flex min-w-0 items-center gap-2">
                  <span className="truncate">{column.header}</span>
                  {column.meta && <span className="shrink-0 text-[10px] uppercase text-[var(--u-color-text-soft)]">{column.meta}</span>}
                </div>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 ? (
            <tr>
              <td
                className="h-20 text-center text-[12px] text-[var(--u-color-text-muted)]"
                colSpan={Math.max(columns.length, 1)}
              >
                {empty ?? "No rows"}
              </td>
            </tr>
          ) : (
            rows.map((row, rowIndex) => (
              <tr
                className="border-b border-[color:color-mix(in_srgb,var(--u-color-border)_62%,transparent)] hover:bg-[var(--u-color-surface-hover)]"
                key={getRowKey?.(row, rowIndex) ?? rowIndex}
              >
                {columns.map((column) => (
                  <td
                    className={cn(
                      "h-[var(--u-size-table-row)] truncate px-2 text-[var(--u-color-text)]",
                      column.align === "right" && "text-right",
                    )}
                    key={column.id}
                    title={String(column.cell?.(row, rowIndex) ?? "")}
                  >
                    {column.cell ? column.cell(row, rowIndex) : null}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
