import { FilePlus2, Pencil, Plug, TerminalSquare } from "lucide-react";
import type {
  SshConnection,
  SshSessionEvent,
  SshSessionSummary,
} from "@unfour/command-client";
import { Button, EmptyState, ErrorState, Tabs, useI18n } from "@unfour/ui";
import type { TerminalSplitMode, TerminalSessionTabState } from "../model/types";
import { formatTerminalError } from "../model/errors";
import { sshEndpointLabel } from "../model/ssh-connection-state";
import { TerminalSearchBar } from "./TerminalSearchBar";
import { TerminalSessionTabMeta } from "./TerminalSessionTab";
import { TerminalSplitView, type TerminalPaneModel } from "./TerminalSplitView";

export function TerminalWorkspace({
  activeSession,
  activeSessionId,
  actionError,
  emptyMessage,
  error,
  events,
  onEditConnection,
  onNewConnection,
  onNewSession,
  onCloseSession,
  onRetry,
  onSelectSession,
  selectedConnection,
  sessions,
  splitMode,
}: {
  activeSession: SshSessionSummary | null;
  activeSessionId: string | null;
  actionError?: unknown;
  emptyMessage: string;
  error?: unknown;
  events: SshSessionEvent[];
  onEditConnection: () => void;
  onNewConnection: () => void;
  onNewSession: () => void;
  onCloseSession: (sessionId: string) => void;
  onRetry: (connectionId: string) => void;
  onSelectSession: (sessionId: string) => void;
  selectedConnection: SshConnection | null;
  sessions: TerminalSessionTabState[];
  splitMode: TerminalSplitMode;
}) {
  const { t } = useI18n();
  const hasSessions = sessions.length > 0;
  const secondaryTab =
    sessions.find(
      (item) =>
        item.session.sessionId !== activeSessionId && item.session.status === "connected",
    ) ??
    sessions.find((item) => item.session.sessionId !== activeSessionId) ??
    null;
  const activeTab =
    sessions.find((item) => item.session.sessionId === activeSession?.sessionId) ?? null;

  const primaryModel: TerminalPaneModel | null = activeSession
    ? {
        connection: activeTab?.connection ?? selectedConnection,
        events: events.filter((event) => event.sessionId === activeSession.sessionId),
        session: activeSession,
      }
    : null;
  const secondaryModel: TerminalPaneModel | null = secondaryTab
    ? {
        connection: secondaryTab.connection,
        events: events.filter(
          (event) => event.sessionId === secondaryTab.session.sessionId,
        ),
        session: secondaryTab.session,
      }
    : null;

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      {hasSessions ? (
        <Tabs
          activeId={activeSessionId ?? sessions[0]?.session.sessionId ?? ""}
          onClose={onCloseSession}
          onSelect={onSelectSession}
          tabs={sessions.map((item) => ({
            id: item.session.sessionId,
            loading:
              item.session.status === "connected" &&
              item.session.sessionId !== activeSessionId,
            meta: <TerminalSessionTabMeta session={item.session} />,
            modified: item.modified,
            title: `${item.session.username}@${item.session.host}`,
          }))}
        />
      ) : null}
      {Boolean(actionError) && (
        <div className="shrink-0 truncate border-b border-[var(--u-color-border)] bg-[var(--u-color-danger-soft)] px-3 py-1 text-[12px] text-[var(--u-color-danger)]">
          {formatTerminalError(actionError)}
        </div>
      )}
      <div className="relative flex min-h-0 flex-1">
        <TerminalSearchBar />
        {error ? (
          <ErrorState className="h-full min-h-0 flex-1 rounded-none border-0">
            {formatTerminalError(error)}
          </ErrorState>
        ) : primaryModel ? (
          <TerminalSplitView
            onRetry={onRetry}
            primary={primaryModel}
            secondary={secondaryModel}
            splitMode={splitMode}
          />
        ) : selectedConnection ? (
          <ReadyToConnectState
            connection={selectedConnection}
            onEditConnection={onEditConnection}
            onNewSession={onNewSession}
          />
        ) : (
          <EmptyState className="h-full min-h-0 flex-1 rounded-none border-0">
            <div className="flex max-w-[520px] flex-col items-center gap-3">
              <div className="space-y-1">
                <div className="text-[13px] font-semibold text-[var(--u-color-text)]">
                  {t("ssh.empty.noSessionOpen")}
                </div>
                <div>{emptyMessage}</div>
              </div>
              <Button onClick={onNewConnection} size="sm" type="button" variant="outline">
                <FilePlus2 size={14} />
                {t("ssh.actions.newConnection")}
              </Button>
            </div>
          </EmptyState>
        )}
      </div>
    </div>
  );
}

function ReadyToConnectState({
  connection,
  onEditConnection,
  onNewSession,
}: {
  connection: SshConnection;
  onEditConnection: () => void;
  onNewSession: () => void;
}) {
  const { t } = useI18n();
  return (
    <EmptyState className="h-full min-h-0 flex-1 rounded-none border-0">
      <div className="flex max-w-[520px] flex-col items-center gap-3">
        <div className="grid h-[52px] w-[52px] place-items-center rounded-[var(--u-radius-lg)] bg-[var(--u-color-primary-soft)] text-[var(--u-color-primary)]">
          <Plug size={24} />
        </div>
        <div className="space-y-1">
          <div className="text-[14px] font-semibold text-[var(--u-color-text)]">
            {t("ssh.pane.readyTitle")}
          </div>
          <div>
            {t("ssh.pane.readyDetail", {
              endpoint: sshEndpointLabel(connection),
              name: connection.name,
            })}
          </div>
        </div>
        <div className="flex flex-wrap justify-center gap-2">
          <Button onClick={onEditConnection} size="sm" type="button" variant="outline">
            <Pencil size={14} />
            {t("ssh.pane.editConnection")}
          </Button>
          <Button onClick={onNewSession} size="sm" type="button">
            <TerminalSquare size={14} />
            {t("ssh.pane.openSession")}
          </Button>
        </div>
      </div>
    </EmptyState>
  );
}
