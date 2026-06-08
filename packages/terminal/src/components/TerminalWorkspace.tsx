import type { SshSessionEvent, SshSessionSummary } from "@unfour/command-client";
import { EmptyState, ErrorState, Tabs } from "@unfour/ui";
import type { TerminalSplitMode, TerminalSessionTabState } from "../model/types";
import { TerminalSearchBar } from "./TerminalSearchBar";
import { TerminalSessionTabMeta } from "./TerminalSessionTab";
import { TerminalSplitView } from "./TerminalSplitView";

export function TerminalWorkspace({
  activeSession,
  activeSessionId,
  emptyMessage,
  error,
  events,
  onCloseSession,
  onSelectSession,
  sessions,
  splitMode,
}: {
  activeSession: SshSessionSummary | null;
  activeSessionId: string | null;
  emptyMessage: string;
  error?: unknown;
  events: SshSessionEvent[];
  onCloseSession: (sessionId: string) => void;
  onSelectSession: (sessionId: string) => void;
  sessions: TerminalSessionTabState[];
  splitMode: TerminalSplitMode;
}) {
  const hasSessions = sessions.length > 0;
  const secondarySession =
    sessions.find(
      (item) =>
        item.session.sessionId !== activeSessionId && item.session.status === "active",
    )?.session ??
    sessions.find((item) => item.session.sessionId !== activeSessionId)?.session ??
    null;

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      {hasSessions ? (
        <Tabs
          activeId={activeSessionId ?? sessions[0]?.session.sessionId ?? ""}
          onClose={onCloseSession}
          onSelect={onSelectSession}
          tabs={sessions.map((item) => ({
            id: item.session.sessionId,
            loading: item.session.status === "active" && item.session.sessionId !== activeSessionId,
            meta: <TerminalSessionTabMeta session={item.session} />,
            modified: item.modified,
            title: item.title,
          }))}
        />
      ) : null}
      <div className="relative flex min-h-0 flex-1">
        <TerminalSearchBar />
        {error ? (
          <ErrorState className="h-full min-h-0 flex-1 rounded-none border-0">
            {formatWorkspaceError(error)}
          </ErrorState>
        ) : hasSessions || activeSession ? (
          <TerminalSplitView
            activeSession={activeSession}
            activeEvents={events.filter(
              (event) => event.sessionId === activeSession?.sessionId,
            )}
            secondaryEvents={events.filter(
              (event) => event.sessionId === secondarySession?.sessionId,
            )}
            secondarySession={secondarySession}
            splitMode={splitMode}
          />
        ) : (
          <EmptyState className="h-full min-h-0 flex-1 rounded-none border-0">
            {emptyMessage}
          </EmptyState>
        )}
      </div>
    </div>
  );
}

function formatWorkspaceError(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}
