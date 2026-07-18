import { Clipboard, Eye, Search, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import type { DatabaseQueryResult, DatabaseTableColumn } from "@unfour/command-client";
import {
  Button,
  DataTable,
  Dialog,
  DialogBody,
  DialogContent,
  DialogHeader,
  DialogTitle,
  IconButton,
  Input,
  Select,
  StatusBadge,
  useI18n,
  type DataTableColumn,
  type DataTableSelection,
} from "@unfour/ui";
import type { TableEditing } from "../model/types";
import { databaseValue, pendingValue } from "../model/table-row-editing";
import { serializeDatabaseCell, serializeDatabaseRow, tryFormatJson } from "../result-utils";
import {
  buildSkeletonColumns,
  buildSkeletonRows,
  compareCells,
  copyStatusLabel,
  renderCell,
  renderServerSortIcon,
  renderSortIcon,
  type SortState,
} from "./table-data-grid-helpers";

const MAX_RENDERED_ROWS = 500;

type CellViewer = { columnName: string; value: string | null };
type DataRow = Array<string | null> & { pendingRowKey?: string };
type EditTarget = { row: DataRow; columnIndex: number };
type ServerControls = {
  sort: { column: string; descending: boolean } | null;
  filter: string;
  onSort: (column: string) => void;
  onFilter: (filter: string) => void;
};

export function TableDataGrid({
  columns,
  editing,
  loading,
  result,
  server,
}: {
  columns?: DatabaseTableColumn[] | null;
  editing?: TableEditing | null;
  loading?: boolean;
  result: DatabaseQueryResult;
  server?: ServerControls | null;
}) {
  const { t } = useI18n();
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied-cell" | "copied-row" | "failed">("idle");
  const [filter, setFilter] = useState("");
  const [sort, setSort] = useState<SortState | null>(null);
  const [activeCell, setActiveCell] = useState<{ row: number; column: number } | null>(null);
  const [viewer, setViewer] = useState<CellViewer | null>(null);
  const [viewerRaw, setViewerRaw] = useState(false);
  const [edit, setEdit] = useState<EditTarget | null>(null);
  const [editValue, setEditValue] = useState("");
  const [editMode, setEditMode] = useState<"value" | "null">("value");
  const [selection, setSelection] = useState<DataTableSelection | null>(null);
  const [columnsWidths, setColumnsWidths] = useState<Record<string, number>>(() => ({
    __row_actions: editing ? 72 : 48,
  }));

  const viewerJson = viewer?.value !== null && viewer ? tryFormatJson(viewer.value) : null;
  const isSkeleton = Boolean(loading && columns?.length);
  const skeletonRows = useMemo<DataRow[]>(
    () => (isSkeleton && columns ? buildSkeletonRows(columns, 12) : []),
    [isSkeleton, columns],
  );
  const skeletonColumns = useMemo<DataTableColumn<DataRow>[]>(
    () => (isSkeleton && columns ? buildSkeletonColumns(columns, columnsWidths) : []),
    [isSkeleton, columns, columnsWidths],
  );
  const insertedRows = useMemo<DataRow[]>(
    () =>
      (editing?.pendingChanges ?? [])
        .filter((change) => change.operation === "insert")
        .map((change) => {
          const row = result.columns.map((column) => {
            const cell = change.values.find((candidate) => candidate.column === column.name);
            if (!cell || cell.mode === "null") return null;
            if (cell.mode === "default") return "DEFAULT";
            return cell.value ?? "";
          }) as DataRow;
          row.pendingRowKey = change.rowKey;
          return row;
        }),
    [editing?.pendingChanges, result.columns],
  );

  const processedRows = useMemo(() => {
    const rows = [...result.rows, ...insertedRows] as DataRow[];
    if (server) return rows;
    const needle = filter.trim().toLowerCase();
    const filtered = needle
      ? rows.filter((row) => row.some((value) => (value ?? "").toLowerCase().includes(needle)))
      : rows;
    if (!sort) return filtered;
    return [...filtered].sort((left, right) => {
      const compared = compareCells(left[sort.columnIndex], right[sort.columnIndex]);
      return sort.direction === "asc" ? compared : -compared;
    });
  }, [filter, insertedRows, result.rows, server, sort]);
  const visibleRows = processedRows.slice(0, MAX_RENDERED_ROWS);
  const hasPendingChanges = Boolean(editing?.pendingChanges.length);

  async function copyText(text: string, status: "copied-cell" | "copied-row") {
    try {
      await navigator.clipboard.writeText(text);
      setCopyStatus(status);
      window.setTimeout(() => setCopyStatus("idle"), 1400);
    } catch {
      setCopyStatus("failed");
    }
  }

  function toggleSort(columnIndex: number) {
    setSort((current) => {
      if (!current || current.columnIndex !== columnIndex) return { columnIndex, direction: "asc" };
      if (current.direction === "asc") return { columnIndex, direction: "desc" };
      return null;
    });
  }

  function commitEdit() {
    if (!edit || !editing) {
      setEdit(null);
      return;
    }
    const columnName = result.columns[edit.columnIndex]?.name;
    if (columnName) {
      const rowKey = edit.row.pendingRowKey ?? editing.rowKey(edit.row);
      const original = edit.row[edit.columnIndex] ?? null;
      const current = pendingValue(editing.pendingChanges, rowKey, columnName, original);
      const next = editMode === "null" ? databaseValue(columnName, null) : databaseValue(columnName, editValue);
      if (next.mode === "null" ? current !== null : current !== next.value) {
        editing.onUpdateCell(edit.row, columnName, next, rowKey);
      }
    }
    setEdit(null);
  }

  const rowActionColumn: DataTableColumn<DataRow> = {
    cell: (row, rowIndex) => {
      const rowKey = row.pendingRowKey ?? editing?.rowKey(row);
      const deleted = editing?.pendingChanges.some(
        (change) => change.rowKey === rowKey && change.operation === "delete",
      );
      return (
        <div className="flex items-center gap-0.5">
          <IconButton
            label={t("database.grid.copyRow", { row: rowIndex + 1 })}
            onClick={() => copyText(serializeDatabaseRow(result, row, "\t"), "copied-row")}
            size="compact"
          >
            <Clipboard size={12} />
          </IconButton>
          {editing ? (
            <IconButton
              disabled={(!editing.canUpdateDelete && !row.pendingRowKey) || editing.pending || deleted}
              label={t("database.editing.deleteRow")}
              onClick={() => editing.onDeleteRow(row, rowKey)}
              size="compact"
            >
              <Trash2 className="text-[var(--u-color-danger)]" size={12} />
            </IconButton>
          ) : null}
        </div>
      );
    },
    header: "#",
    id: "__row_actions",
    width: columnsWidths.__row_actions ?? (editing ? 72 : 48),
  };

  const dataColumns: DataTableColumn<DataRow>[] = [
    rowActionColumn,
    ...result.columns.map((column, columnIndex) => ({
      cell: (row: DataRow, rowIndex: number) => {
        const rowKey = row.pendingRowKey ?? editing?.rowKey(row) ?? JSON.stringify(row);
        const change = editing?.pendingChanges.find((candidate) => candidate.rowKey === rowKey);
        const value = pendingValue(editing?.pendingChanges ?? [], rowKey, column.name, row[columnIndex] ?? null);
        const changed = change?.values.some((cell) => cell.column === column.name);
        const deleted = change?.operation === "delete";
        const active = activeCell?.row === rowIndex && activeCell.column === columnIndex;
        if (edit?.row === row && edit.columnIndex === columnIndex) {
          return (
            <div
              className="flex min-w-[180px] items-center gap-1"
              onBlur={(event) => {
                if (!event.currentTarget.contains(event.relatedTarget)) commitEdit();
              }}
            >
              <Select
                aria-label={t("database.editing.valueMode")}
                className="h-6 w-[76px]"
                onChange={(event) => setEditMode(event.target.value as "value" | "null")}
                options={[
                  { label: t("database.editing.value"), value: "value" },
                  { label: "NULL", value: "null" },
                ]}
                value={editMode}
              />
              <input
                autoFocus
                className="block min-w-0 flex-1 rounded-sm border border-[var(--u-color-focus)] bg-[var(--u-color-surface)] px-1 font-mono text-[12px] text-[var(--u-color-text)] focus-visible:outline-none disabled:opacity-50"
                disabled={editMode === "null"}
                onChange={(event) => setEditValue(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    commitEdit();
                  } else if (event.key === "Escape") {
                    event.preventDefault();
                    setEdit(null);
                  }
                }}
                value={editValue}
              />
            </div>
          );
        }
        return (
          <button
            className={[
              "block w-full cursor-pointer truncate text-left font-mono text-[12px] text-[var(--u-color-text)] focus-visible:outline-none",
              active ? "ring-1 ring-inset ring-[var(--u-color-focus)]" : "focus-visible:ring-1 focus-visible:ring-[var(--u-color-focus)]",
              changed ? "bg-[color:color-mix(in_srgb,var(--u-color-warning)_15%,transparent)]" : "",
              deleted ? "line-through opacity-45" : "",
            ].join(" ")}
            onClick={() => setActiveCell({ row: rowIndex, column: columnIndex })}
            onDoubleClick={
              editing &&
              (editing.canUpdateDelete || Boolean(row.pendingRowKey)) &&
              !columns?.find((candidate) => candidate.name === column.name)?.generated &&
              !editing.pending &&
              !deleted
                ? () => {
                    setEditValue(value ?? "");
                    setEditMode(value === null ? "null" : "value");
                    setEdit({ row, columnIndex });
                  }
                : undefined
            }
            title={
              editing && (editing.canUpdateDelete || row.pendingRowKey)
                ? t("database.editing.editHint")
                : (value ?? "NULL")
            }
            type="button"
          >
            {renderCell(value)}
          </button>
        );
      },
      header: (
        <button
          className="group/header flex w-full min-w-0 cursor-pointer items-center gap-1 text-left hover:text-[var(--u-color-text)] focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          disabled={hasPendingChanges}
          onClick={() => (server ? server.onSort(column.name) : toggleSort(columnIndex))}
          title={t("database.grid.sortBy", { column: column.name })}
          type="button"
        >
          <span className="truncate">{column.name}</span>
          {server ? renderServerSortIcon(server.sort, column.name) : renderSortIcon(sort, columnIndex)}
        </button>
      ),
      id: column.name || `column-${columnIndex}`,
      meta: column.dataType,
      width: columnsWidths[column.name || `column-${columnIndex}`] ?? Math.min(Math.max(column.name.length * 9 + 96, 140), 360),
    })),
  ];

  const gridColumns = isSkeleton ? skeletonColumns : dataColumns;
  const gridRows = isSkeleton ? skeletonRows : visibleRows;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex h-8 shrink-0 items-center gap-2 border-b border-[var(--u-color-border)] px-2">
        <Search className="text-[var(--u-color-text-soft)]" size={13} />
        <Input
          aria-label={t("database.grid.filterPlaceholder")}
          className="h-6 max-w-[260px]"
          disabled={isSkeleton || hasPendingChanges}
          onChange={(event) => (server ? server.onFilter(event.target.value) : setFilter(event.target.value))}
          placeholder={t("database.grid.filterPlaceholder")}
          value={server ? server.filter : filter}
        />
        {!server && sort ? (
          <button
            className="text-[11px] text-[var(--u-color-text-soft)] hover:text-[var(--u-color-text)]"
            onClick={() => setSort(null)}
            type="button"
          >
            {t("database.grid.clearSort")}
          </button>
        ) : null}
      </div>
      <DataTable
        className="flex-1"
        columns={gridColumns}
        empty={(server ? server.filter : filter) ? t("database.grid.noMatches") : t("database.grid.empty")}
        getRowKey={(row, index) => row.pendingRowKey ?? editing?.rowKey(row) ?? `${JSON.stringify(row)}:${index}`}
        onColumnResize={(columnId, width) => setColumnsWidths((current) => ({ ...current, [columnId]: width }))}
        onSelectionChange={setSelection}
        rows={gridRows}
        selection={selection}
      />
      <div className="flex h-7 shrink-0 items-center justify-between border-t border-[var(--u-color-border)] px-2 text-[11px] text-[var(--u-color-text-soft)]">
        <div className="flex items-center gap-1">
          <span>{copyStatusLabel(copyStatus, t)}</span>
          {activeCell && visibleRows[activeCell.row] && result.columns[activeCell.column] ? (
            <IconButton
              label={t("database.grid.valueViewer")}
              onClick={() => {
                const row = visibleRows[activeCell.row];
                const column = result.columns[activeCell.column];
                const rowKey = row.pendingRowKey ?? editing?.rowKey(row) ?? JSON.stringify(row);
                setViewerRaw(false);
                setViewer({
                  columnName: column.name,
                  value: pendingValue(editing?.pendingChanges ?? [], rowKey, column.name, row[activeCell.column] ?? null),
                });
              }}
              size="compact"
            >
              <Eye size={12} />
            </IconButton>
          ) : null}
        </div>
        {!isSkeleton ? (
          <span>
            {processedRows.length > visibleRows.length
              ? t("database.grid.showingFirst", { shown: visibleRows.length, total: processedRows.length })
              : t("database.grid.rowsRendered", { count: processedRows.length })}
          </span>
        ) : null}
      </div>
      <Dialog onOpenChange={(open) => !open && setViewer(null)} open={viewer !== null}>
        <DialogContent title={t("database.grid.valueViewer")}>
          <DialogHeader>
            <DialogTitle>{viewer?.columnName ?? t("database.grid.valueViewer")}</DialogTitle>
          </DialogHeader>
          <DialogBody className="space-y-2">
            {viewer?.value === null ? (
              <StatusBadge>NULL</StatusBadge>
            ) : (
              <>
                {viewerJson?.isJson ? (
                  <div className="flex items-center justify-between">
                    <StatusBadge>JSON</StatusBadge>
                    <button
                      className="text-[11px] text-[var(--u-color-text-soft)] hover:text-[var(--u-color-text)]"
                      onClick={() => setViewerRaw((current) => !current)}
                      type="button"
                    >
                      {viewerRaw ? t("database.grid.viewFormatted") : t("database.grid.viewRaw")}
                    </button>
                  </div>
                ) : null}
                <div className="max-h-[50vh] overflow-auto rounded border border-[var(--u-color-border)] bg-[var(--u-color-surface-subtle)]">
                  <pre className="whitespace-pre-wrap break-words p-2 font-mono text-[12px] text-[var(--u-color-text)]">
                    {viewerJson?.isJson && !viewerRaw ? viewerJson.formatted : viewer?.value}
                  </pre>
                </div>
              </>
            )}
            <div className="flex items-center justify-between">
              <span className="text-[11px] text-[var(--u-color-text-soft)]">{t("database.grid.selectToCopy")}</span>
              <Button
                disabled={viewer?.value == null}
                onClick={() => viewer?.value != null && copyText(serializeDatabaseCell(viewer.value, "\t"), "copied-cell")}
                size="sm"
                type="button"
                variant="outline"
              >
                <Clipboard size={13} />
                {t("database.grid.copyValue")}
              </Button>
            </div>
          </DialogBody>
        </DialogContent>
      </Dialog>
    </div>
  );
}
