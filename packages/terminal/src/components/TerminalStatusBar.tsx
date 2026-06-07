import { useMemo } from "react";
import { StatusBar } from "@unfour/ui";
import { useWorkspaceStore } from "@unfour/workspace";
import { useSshConnections } from "../hooks/useSshConnections";
import { useTerminalSessions } from "../hooks/useTerminalSessions";
import { useTerminalStore } from "../model/terminal-state";
import { sshEndpointLabel } from "../model/ssh-connection-state";

export function TerminalStatusBar({
  workspaceId,
  workspaceName,
}: {
  workspaceId: string;
  workspaceName: string;
}) {
  const selectedSshConnectionId = useWorkspaceStore((state) => state.selectedSshConnectionId);
  const activeSessionId = useTerminalStore((state) => state.activeSessionId);
  const connectionsQuery = useSshConnections(workspaceId);
  const sessionsQuery = useTerminalSessions(workspaceId);
  const selectedConnection = useMemo(
    () => connectionsQuery.data?.find((item) => item.id === selectedSshConnectionId) ?? null,
    [connectionsQuery.data, selectedSshConnectionId],
  );
  const activeSession = useMemo(
    () => sessionsQuery.data?.find((item) => item.sessionId === activeSessionId) ?? null,
    [activeSessionId, sessionsQuery.data],
  );

  return (
    <StatusBar>
      <div className="flex min-w-0 items-center gap-3">
        <span className="truncate">{workspaceName}</span>
        <span className="truncate">{sshEndpointLabel(selectedConnection)}</span>
        <span>{activeSession?.status ?? "disconnected"}</span>
      </div>
      <div className="flex shrink-0 items-center gap-3">
        <span>{activeSession ? `${activeSession.cols}x${activeSession.rows}` : "no pty"}</span>
        <span>{selectedConnection?.authKind ?? "no auth"}</span>
      </div>
    </StatusBar>
  );
}
