import { useQuery } from "@tanstack/react-query";
import { listSshSessions } from "@unfour/command-client";

export function useTerminalSessions(
  workspaceId: string,
  options?: { active?: boolean },
) {
  const active = options?.active ?? true;
  return useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["ssh-sessions", workspaceId],
    queryFn: () => listSshSessions(workspaceId),
    // Keep Connections mounted under Tasks for draft/session continuity, but
    // stop the 2s poll while that surface is hidden so idle Tasks stays cheap.
    refetchInterval: active ? 2_000 : false,
  });
}
