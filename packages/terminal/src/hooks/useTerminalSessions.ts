import { useQuery } from "@tanstack/react-query";
import { listSshSessions } from "@unfour/command-client";

export function useTerminalSessions(workspaceId: string) {
  return useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["ssh-sessions", workspaceId],
    queryFn: () => listSshSessions(workspaceId),
    refetchInterval: 2_000,
  });
}
