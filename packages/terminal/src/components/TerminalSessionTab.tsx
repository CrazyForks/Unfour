import type { SshSessionSummary } from "@unfour/command-client";
import { ConnectionStatus } from "@unfour/ui";
import { terminalSessionStatus } from "../model/terminal-session-status";

export function TerminalSessionTabMeta({ session }: { session: SshSessionSummary }) {
  return (
    <ConnectionStatus
      label={
        session.status === "reconnecting"
          ? `reconnecting ${session.reconnectAttempt}/3`
          : session.status
      }
      status={terminalSessionStatus(session)}
    />
  );
}
