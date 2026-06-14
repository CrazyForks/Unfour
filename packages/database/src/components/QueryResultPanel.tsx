import { Clipboard, Download } from "lucide-react";
import { useState } from "react";
import type { DatabaseQueryResult } from "@unfour/command-client";
import { Button, EmptyState, ErrorState, LoadingState, Tabs, Toolbar, ToolbarGroup } from "@unfour/ui";
import { confirmationMessage, formatDatabaseError, serializeDatabaseResult } from "../result-utils";
import { TableDataGrid } from "./TableDataGrid";

export function QueryResultPanel({
  activeTab,
  error,
  isPending,
  onSelectTab,
  pendingConfirmation,
  result,
}: {
  activeTab: "results" | "messages" | "logs";
  error: unknown;
  isPending: boolean;
  onSelectTab: (tab: "results" | "messages" | "logs") => void;
  pendingConfirmation: boolean;
  result: DatabaseQueryResult | null;
}) {
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "failed">("idle");
  const [lastResult, setLastResult] = useState(result);
  if (result !== lastResult) {
    setLastResult(result);
    setCopyStatus("idle");
  }

  async function copyTsv() {
    if (!result) {
      return;
    }
    try {
      await navigator.clipboard.writeText(serializeDatabaseResult(result, "\t"));
      setCopyStatus("copied");
      window.setTimeout(() => setCopyStatus("idle"), 1600);
    } catch {
      setCopyStatus("failed");
    }
  }

  function exportCsv() {
    if (!result) {
      return;
    }
    const blob = new Blob([serializeDatabaseResult(result, ",")], { type: "text/csv;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `unfour-query-results-${new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-")}.csv`;
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
  }

  return (
    <section className="flex min-h-[190px] flex-[0.45] flex-col border-t border-[var(--u-color-border)] bg-[var(--u-color-surface)]">
      <Tabs
        activeId={activeTab}
        className="h-[30px]"
        onSelect={(tabId) => onSelectTab(tabId as "results" | "messages" | "logs")}
        tabs={[
          { id: "results", title: "Results" },
          { id: "messages", title: "Messages" },
          { id: "logs", title: "Logs" },
        ]}
      />
      <Toolbar className="h-8">
        <ToolbarGroup>
          <span className="text-[12px] text-[var(--u-color-text-muted)]">
            {result ? `${result.rows.length} rows in ${result.durationMs}ms` : "No execution yet"}
          </span>
        </ToolbarGroup>
        <ToolbarGroup>
          <Button disabled={!result} onClick={copyTsv} size="sm" type="button" variant="outline">
            <Clipboard size={13} />
            {copyStatus === "copied" ? "Copied" : copyStatus === "failed" ? "Copy failed" : "Copy TSV"}
          </Button>
          <Button disabled={!result} onClick={exportCsv} size="sm" type="button" variant="outline">
            <Download size={13} />
            Export CSV
          </Button>
        </ToolbarGroup>
      </Toolbar>
      {activeTab === "results" && renderResults({ error, isPending, pendingConfirmation, result })}
      {activeTab === "messages" && <Messages result={result} />}
      {activeTab === "logs" && <Logs error={error} isPending={isPending} result={result} />}
    </section>
  );
}

function renderResults({
  error,
  isPending,
  pendingConfirmation,
  result,
}: {
  error: unknown;
  isPending: boolean;
  pendingConfirmation: boolean;
  result: DatabaseQueryResult | null;
}) {
  if (error) {
    return (
      <ErrorState className="m-2 min-h-0 flex-1">
        {pendingConfirmation ? confirmationMessage(error) : formatDatabaseError(error)}
      </ErrorState>
    );
  }

  if (isPending) {
    return <LoadingState className="m-2 min-h-0 flex-1">Running query...</LoadingState>;
  }

  if (!result) {
    return <EmptyState className="m-2 min-h-0 flex-1">Query results will appear here.</EmptyState>;
  }

  if (result.columns.length === 0) {
    return (
      <EmptyState className="m-2 min-h-0 flex-1">
        {result.affectedRows} rows affected in {result.durationMs}ms.
      </EmptyState>
    );
  }

  return <TableDataGrid result={result} />;
}

function Messages({ result }: { result: DatabaseQueryResult | null }) {
  return (
    <div className="min-h-0 flex-1 overflow-auto p-2 text-[12px] text-[var(--u-color-text-muted)]">
      {result ? `${result.affectedRows} affected rows. Safety: ${result.safety.classification}.` : "No messages."}
    </div>
  );
}

function Logs({
  error,
  isPending,
  result,
}: {
  error: unknown;
  isPending: boolean;
  result: DatabaseQueryResult | null;
}) {
  return (
    <pre className="min-h-0 flex-1 overflow-auto p-2 font-mono text-[12px] text-[var(--u-color-text-muted)]">
      {isPending
        ? "Executing SQL..."
        : error
          ? formatDatabaseError(error)
          : result
            ? `duration=${result.durationMs}ms rows=${result.rows.length} affected=${result.affectedRows}`
            : "No logs."}
    </pre>
  );
}
