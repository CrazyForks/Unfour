import type { SshConnection, SshSessionSummary } from "@unfour/command-client";
import type { TerminalSessionTabState } from "./types";

export function buildTerminalSessionTabs({
  connections,
  sessions,
}: {
  connections: SshConnection[];
  sessions: SshSessionSummary[];
}): TerminalSessionTabState[] {
  const titleCounts = new Map<string, number>();

  return sessions.map((session) => {
    const connection = connections.find((item) => item.id === session.connectionId) ?? null;
    const baseTitle = connection?.name ?? `${session.username}@${session.host}`;
    const count = titleCounts.get(baseTitle) ?? 0;
    titleCounts.set(baseTitle, count + 1);

    return {
      connection,
      session,
      title: count === 0 ? baseTitle : `${baseTitle} ${count + 1}`,
    };
  });
}
