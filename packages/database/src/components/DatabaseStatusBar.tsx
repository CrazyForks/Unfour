import type { DatabaseConnection } from "@unfour/command-client";

export function DatabaseStatusBar({
  connection,
  executing,
}: {
  connection: DatabaseConnection | null;
  executing: boolean;
}) {
  return (
    <div className="flex h-[var(--u-size-statusbar)] items-center justify-between border-t border-[var(--u-color-border)] bg-[var(--u-color-surface-muted)] px-2 text-[12px] text-[var(--u-color-text-muted)]">
      <span className="truncate">{connection ? connection.name : "No database connection"}</span>
      <span>{executing ? "SQL executing" : "Ready"}</span>
    </div>
  );
}
