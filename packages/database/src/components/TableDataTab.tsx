import { ChevronLeft, ChevronRight, Database, Plus, RefreshCw } from "lucide-react";
import { useState } from "react";
import type { DatabaseCellValue, DatabaseQueryResult, DatabaseTable } from "@unfour/command-client";
import {
  Button,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  EmptyState,
  ErrorState,
  IconButton,
  Input,
  Select,
  Toolbar,
  ToolbarGroup,
  useI18n,
} from "@unfour/ui";
import type { DatabaseTableViewState, TableEditing } from "../model/types";
import { buildPreviewSql } from "../result-utils";
import { DatabaseErrorDetails } from "./DatabaseErrorDetails";
import { TableDataGrid } from "./TableDataGrid";

export function TableDataTab({
  editing,
  error,
  executePending,
  loading,
  onPageChange,
  onRefresh,
  onSwitchToStructure,
  onTableFilter,
  onTableSort,
  result,
  table,
  tableFilter,
  tableSort,
  tableView,
}: {
  editing?: TableEditing | null;
  error?: unknown;
  executePending: boolean;
  /** True while the table data is being fetched (tab.loading). */
  loading?: boolean;
  onPageChange: (pageIndex: number, pageSize: number) => void;
  onRefresh: () => void;
  onSwitchToStructure?: () => void;
  onTableFilter: (filter: string) => void;
  onTableSort: (column: string) => void;
  result: DatabaseQueryResult | null;
  /** Schema of the opened table; used to paint headers immediately while loading. */
  table?: DatabaseTable | null;
  tableFilter: string;
  tableSort: { column: string; descending: boolean } | null;
  tableView: DatabaseTableViewState | null;
}) {
  const { t } = useI18n();
  const [addOpen, setAddOpen] = useState(false);

  if (error) {
    return (
      <ErrorState className="m-2 min-h-0 flex-1">
        <DatabaseErrorDetails error={error} />
      </ErrorState>
    );
  }

  const isLoading = Boolean(loading && (!result || !tableView));
  const tableMeta = table ?? null;

  // Loading with a known schema: render the shell + skeleton grid instead of
  // bouncing between an empty placeholder and the full grid. Without a schema
  // (or when not loading) fall back to the real empty state.
  if (!result && !tableView && !isLoading) {
    return <EmptyState className="m-2 min-h-0 flex-1">{t("database.editor.tableDataEmpty")}</EmptyState>;
  }

  const displayName = tableView?.tableName ?? tableMeta?.name ?? "";
  const gridResult: DatabaseQueryResult = result ?? buildSkeletonResult(tableMeta);

  const firstRow = tableView && tableView.totalRows > 0 ? tableView.pageIndex * tableView.pageSize + 1 : 0;
  const lastRow = tableView ? Math.min(tableView.totalRows, (tableView.pageIndex + 1) * tableView.pageSize) : 0;
  const hasPrevious = Boolean(tableView && tableView.pageIndex > 0);
  const hasNext = Boolean(tableView && lastRow < tableView.totalRows);
  const totalPages = tableView ? Math.max(1, Math.ceil(tableView.totalRows / Math.max(1, tableView.pageSize))) : 1;
  const previewSql = tableView ? buildPreviewSql(tableView.tableName, tableView.pageSize, tableView.pageIndex) : "";

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <Toolbar className="h-8">
        <ToolbarGroup className="min-w-0">
          <span className="truncate text-[12px] font-medium text-[var(--u-color-text)]">{displayName}</span>
          {!isLoading && tableView ? (
            <span className="text-[11px] text-[var(--u-color-text-soft)]">
              {firstRow}-{lastRow} of {tableView.totalRows}
            </span>
          ) : null}
        </ToolbarGroup>
        <ToolbarGroup>
          {onSwitchToStructure ? (
            <button
              className="inline-flex h-[22px] items-center rounded-[5px] px-2 text-[12px] font-medium text-[var(--u-color-text-muted)] transition-colors duration-150 hover:text-[var(--u-color-text)]"
              onClick={onSwitchToStructure}
              type="button"
            >
              {t("database.editor.structureView")}
            </button>
          ) : null}
          {editing ? (
            <Button
              disabled={editing.pending || isLoading}
              onClick={() => setAddOpen(true)}
              size="sm"
              type="button"
              variant="outline"
            >
              <Plus size={13} />
              {t("database.editing.addRow")}
            </Button>
          ) : null}
          <Select
            aria-label={t("database.grid.pageSizeAria")}
            className="w-[88px]"
            disabled={executePending || isLoading}
            onChange={(event) => onPageChange(0, Number(event.target.value))}
            options={[
              { label: "50", value: "50" },
              { label: "100", value: "100" },
              { label: "250", value: "250" },
            ]}
            value={tableView ? String(tableView.pageSize) : "50"}
          />
          <IconButton disabled={executePending || isLoading} label={t("database.grid.refreshPreview")} onClick={onRefresh}>
            <RefreshCw size={13} />
          </IconButton>
        </ToolbarGroup>
      </Toolbar>
      {isLoading && (
        <div className="h-0.5 w-full shrink-0 overflow-hidden bg-[var(--u-color-border)]">
          <div className="h-full w-1/3 animate-indeterminate rounded-full bg-[var(--u-color-primary)]" />
        </div>
      )}
      <div className="flex min-h-0 flex-1 flex-col animate-fade-in" key={isLoading ? "skeleton" : "ready"}>
        <TableDataGrid
          columns={tableMeta?.columns}
          editing={editing}
          loading={isLoading}
          result={gridResult}
          server={{
            filter: tableFilter,
            onFilter: onTableFilter,
            onSort: onTableSort,
            sort: tableSort,
          }}
        />
      </div>
      <div className="flex h-9 shrink-0 items-center gap-3 border-t border-[var(--u-color-border)] bg-[var(--u-color-surface)] px-3">
        <Database className="shrink-0 text-[var(--u-color-text-soft)]" size={13} />
        <code
          className="min-w-0 flex-1 truncate font-mono text-[12px] text-[var(--u-color-text-muted)]"
          title={previewSql}
        >
          {previewSql}
        </code>
        {tableView ? (
          <div className="flex shrink-0 items-center gap-1 text-[var(--u-color-text-soft)]">
            <IconButton
              disabled={!hasPrevious || executePending}
              label={t("database.grid.prevPage")}
              onClick={() => onPageChange(tableView.pageIndex - 1, tableView.pageSize)}
              size="compact"
            >
              <ChevronLeft size={14} />
            </IconButton>
            <span className="px-1 text-[11px] tabular-nums text-[var(--u-color-text-muted)]">
              {t("database.grid.page", { page: tableView.pageIndex + 1, pages: totalPages })}
            </span>
            <IconButton
              disabled={!hasNext || executePending}
              label={t("database.grid.nextPage")}
              onClick={() => onPageChange(tableView.pageIndex + 1, tableView.pageSize)}
              size="compact"
            >
              <ChevronRight size={14} />
            </IconButton>
          </div>
        ) : null}
      </div>
      {editing ? (
        <AddRowDialog
          columns={result?.columns.map((column) => column.name) ?? []}
          onOpenChange={setAddOpen}
          onSubmit={(values) => {
            editing.onInsertRow(values);
            setAddOpen(false);
          }}
          open={addOpen}
          pending={editing.pending}
        />
      ) : null}
    </div>
  );
}

function buildSkeletonResult(tableMeta: DatabaseTable | null): DatabaseQueryResult {
  // A result-shaped placeholder so the grid can paint real headers/rows while
  // the real data streams in. The grid uses `columns`/`loading` to render the
  // shimmer rows; the actual row array here is unused.
  const cols = tableMeta?.columns ?? [];
  return {
    columns: cols.map((column) => ({ name: column.name, dataType: column.dataType })),
    rows: [],
    affectedRows: 0,
    durationMs: 0,
    safety: { classification: "select", requiresConfirmation: false, confirmed: false, message: null },
  };
}

function AddRowDialog({
  columns,
  onOpenChange,
  onSubmit,
  open,
  pending,
}: {
  columns: string[];
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: DatabaseCellValue[]) => void;
  open: boolean;
  pending: boolean;
}) {
  const { t } = useI18n();
  const [draft, setDraft] = useState<Record<string, string>>({});

  function submit() {
    // Empty inputs are sent as NULL; non-empty inputs as the typed text.
    const values: DatabaseCellValue[] = columns.map((column) => ({
      column,
      value: draft[column]?.length ? draft[column] : null,
    }));
    onSubmit(values);
    setDraft({});
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent title={t("database.editing.addRow")}>
        <DialogHeader>
          <DialogTitle>{t("database.editing.addRow")}</DialogTitle>
        </DialogHeader>
        <DialogBody className="space-y-2">
          {columns.map((column) => (
            <label className="block space-y-1" key={column}>
              <span className="text-[11px] font-medium uppercase text-[var(--u-color-text-soft)]">{column}</span>
              <Input
                onChange={(event) => setDraft((current) => ({ ...current, [column]: event.target.value }))}
                placeholder={t("database.editing.nullPlaceholder")}
                value={draft[column] ?? ""}
              />
            </label>
          ))}
        </DialogBody>
        <DialogFooter>
          <Button onClick={() => onOpenChange(false)} size="sm" type="button" variant="ghost">
            {t("common.confirm.cancel")}
          </Button>
          <Button disabled={pending} onClick={submit} size="sm" type="button">
            {t("database.editing.insert")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
