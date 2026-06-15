import type { SshSessionSummary } from "@unfour/command-client";

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
