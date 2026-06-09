import type { SshSessionSummary } from "@unfour/command-client";
import { ConnectionStatus } from "@unfour/ui";

export function terminalSessionStatus(session: SshSessionSummary | null | undefined) {
  if (!session) {
    return "disconnected" as const;
  }

  if (session.status === "failed") {
    return "error" as const;
  }
  if (session.status === "degraded" || session.status === "reconnecting") {
    return "connecting" as const;
  }
  return session.status;
}

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
