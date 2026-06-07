import type { SshSessionSummary } from "@unfour/command-client";
import { ConnectionStatus } from "@unfour/ui";

export function terminalSessionStatus(session: SshSessionSummary | null | undefined) {
  if (!session) {
    return "disconnected" as const;
  }

  return session.status === "active" ? "connected" : "closed";
}

export function TerminalSessionTabMeta({ session }: { session: SshSessionSummary }) {
  return (
    <ConnectionStatus
      label={session.status === "active" ? "on" : "off"}
      status={terminalSessionStatus(session)}
    />
  );
}
