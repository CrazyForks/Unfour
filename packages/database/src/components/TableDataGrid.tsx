import { ArrowDown, ArrowUp, ChevronsUpDown, Clipboard, Search } from "lucide-react";
import { useMemo, useState } from "react";
import type { DatabaseQueryResult } from "@unfour/command-client";
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
  StatusBadge,
  useI18n,
  type DataTableColumn,
} from "@unfour/ui";
import { serializeDatabaseCell, serializeDatabaseRow } from "../result-utils";

const MAX_RENDERED_ROWS = 500;

type SortState = { columnIndex: number; direction: "asc" | "desc" };
type CellViewer = { columnName: string; value: string | null };

export function TableDataGrid({ result }: { result: DatabaseQueryResult }) {
  const { t } = useI18n();
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied-cell" | "copied-row" | "failed">("idle");
  const [filter, setFilter] = useState("");
  const [sort, setSort] = useState<SortState | null>(null);
  const [viewer, setViewer] = useState<CellViewer | null>(null);

  const processedRows = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    const filtered = needle
      ? result.rows.filter((row) => row.some((value) => (value ?? "").toLowerCase().includes(needle)))
      : result.rows;

    if (!sort) {
      return filtered;
    }

    // Copy before sorting so the source result order is preserved.
    const sorted = [...filtered].sort((left, right) => {
      const a = left[sort.columnIndex];
      const b = right[sort.columnIndex];
      const compared = compareCells(a, b);
      return sort.direction === "asc" ? compared : -compared;
    });
    return sorted;
  }, [filter, result.rows, sort]);

  const visibleRows = processedRows.slice(0, MAX_RENDERED_ROWS);

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
      if (!current || current.columnIndex !== columnIndex) {
        return { columnIndex, direction: "asc" };
      }
      if (current.direction === "asc") {
        return { columnIndex, direction: "desc" };
      }
      return null;
    });
  }

  const rowActionColumn: DataTableColumn<Array<string | null>> = {
    cell: (row, rowIndex) => (
      <IconButton
        label={`Copy row ${rowIndex + 1}`}
        onClick={() => copyText(serializeDatabaseRow(result, row, "\t"), "copied-row")}
        size="compact"
      >
        <Clipboard size={12} />
      </IconButton>
    ),
    header: "#",
    id: "__row_actions",
    width: 48,
  };

  const columns: DataTableColumn<Array<string | null>>[] = [
    rowActionColumn,
    ...result.columns.map((column, columnIndex) => ({
      cell: (row: Array<string | null>) => {
        const value = row[columnIndex];
        return (
          <button
            className="block w-full cursor-pointer truncate text-left font-mono text-[12px] text-[var(--u-color-text)] hover:text-[var(--u-color-primary)] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--u-color-focus)]"
            onClick={() => setViewer({ columnName: column.name, value: value ?? null })}
            title={value ?? "NULL"}
            type="button"
          >
            {renderCell(value)}
          </button>
        );
      },
      header: (
        <button
          className="flex w-full min-w-0 cursor-pointer items-center gap-1 text-left hover:text-[var(--u-color-text)] focus-visible:outline-none"
          onClick={() => toggleSort(columnIndex)}
          title={t("database.grid.sortBy", { column: column.name })}
          type="button"
        >
          <span className="truncate">{column.name}</span>
          {renderSortIcon(sort, columnIndex)}
        </button>
      ),
      id: column.name || `column-${columnIndex}`,
      meta: column.dataType,
      width: Math.min(Math.max(column.name.length * 9 + 96, 140), 360),
    })),
  ];

  const totalAfterFilter = processedRows.length;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex h-8 shrink-0 items-center gap-2 border-b border-[var(--u-color-border)] px-2">
        <Search className="text-[var(--u-color-text-soft)]" size={13} />
        <Input
          aria-label={t("database.grid.filterPlaceholder")}
          className="h-6 max-w-[260px]"
          onChange={(event) => setFilter(event.target.value)}
          placeholder={t("database.grid.filterPlaceholder")}
          value={filter}
        />
        {sort ? (
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
        columns={columns}
        empty={filter ? t("database.grid.noMatches") : t("database.grid.empty")}
        getRowKey={(_, index) => index}
        rows={visibleRows}
      />
      <div className="flex h-7 shrink-0 items-center justify-between border-t border-[var(--u-color-border)] px-2 text-[11px] text-[var(--u-color-text-soft)]">
        <span>{copyStatusLabel(copyStatus, t)}</span>
        <span>
          {totalAfterFilter > visibleRows.length
            ? t("database.grid.showingFirst", { shown: visibleRows.length, total: totalAfterFilter })
            : t("database.grid.rowsRendered", { count: totalAfterFilter })}
        </span>
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
              <pre className="max-h-[50vh] overflow-auto whitespace-pre-wrap break-words rounded border border-[var(--u-color-border)] bg-[var(--u-color-surface-subtle)] p-2 font-mono text-[12px] text-[var(--u-color-text)]">
                {viewer?.value}
              </pre>
            )}
            <div className="flex justify-end">
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

function compareCells(a: string | null, b: string | null) {
  // NULLs always sort to the end regardless of direction.
  if (a === null && b === null) return 0;
  if (a === null) return 1;
  if (b === null) return -1;
  const numA = Number(a);
  const numB = Number(b);
  if (!Number.isNaN(numA) && !Number.isNaN(numB) && a.trim() !== "" && b.trim() !== "") {
    return numA - numB;
  }
  return a.localeCompare(b);
}

function renderSortIcon(sort: SortState | null, columnIndex: number) {
  if (!sort || sort.columnIndex !== columnIndex) {
    return <ChevronsUpDown className="shrink-0 text-[var(--u-color-text-soft)]" size={12} />;
  }
  return sort.direction === "asc" ? (
    <ArrowUp className="shrink-0 text-[var(--u-color-primary)]" size={12} />
  ) : (
    <ArrowDown className="shrink-0 text-[var(--u-color-primary)]" size={12} />
  );
}

function renderCell(value: string | null | undefined) {
  if (value === null || value === undefined) {
    return <StatusBadge>NULL</StatusBadge>;
  }

  if (value.length > 240) {
    return `${value.slice(0, 240)}...`;
  }

  return value;
}

function copyStatusLabel(
  status: "idle" | "copied-cell" | "copied-row" | "failed",
  t: ReturnType<typeof useI18n>["t"],
) {
  switch (status) {
    case "copied-cell":
      return t("database.grid.cellCopied");
    case "copied-row":
      return t("database.grid.rowCopied");
    case "failed":
      return t("database.grid.copyFailed");
    default:
      return t("database.grid.copyHint");
  }
}
