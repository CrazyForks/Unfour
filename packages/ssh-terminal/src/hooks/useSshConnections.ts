import { useQuery } from "@tanstack/react-query";
import { listSshConnections } from "@unfour/command-client";

export function useSshConnections(workspaceId: string) {
  return useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["ssh-connections", workspaceId],
    queryFn: () => listSshConnections(workspaceId),
  });
}
