import type { SshConnection, SshSessionSummary } from "@unfour/command-client";
import { ConnectionStatus, StatusBadge, useI18n } from "@unfour/ui";
import {
  terminalSessionStatus,
  terminalSessionStatusLabel,
} from "../model/terminal-session-status";

export function TerminalPaneHeader({
  connection,
  session,
}: {
  connection?: SshConnection | null;
  session: SshSessionSummary;
}) {
  const { t } = useI18n();
  const name = connection?.name ?? session.host;
  const authKind = connection?.authKind ?? session.authKind ?? t("ssh.status.noAuth");

  return (
    <div className="flex h-[30px] shrink-0 items-center gap-2.5 border-b border-[var(--u-color-border)] bg-[var(--u-color-surface-subtle)] pl-3 pr-1.5">
      <span className="shrink-0 truncate text-[12px] font-semibold text-[var(--u-color-text)]">
        {name}
      </span>
      <span className="min-w-0 flex-1 truncate font-mono text-[11px] text-[var(--u-color-text-muted)]">
        {session.username}@{session.host}
      </span>
      <div className="flex shrink-0 items-center gap-2">
        <ConnectionStatus
          label={terminalSessionStatusLabel(session, t)}
          status={terminalSessionStatus(session)}
          variant="dot"
        />
        <StatusBadge>
          {session.cols}×{session.rows}
        </StatusBadge>
        <StatusBadge>{authKind}</StatusBadge>
      </div>
    </div>
  );
}
