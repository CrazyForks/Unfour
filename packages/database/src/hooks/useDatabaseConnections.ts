import { useQuery } from "@tanstack/react-query";
import { listDatabaseConnections } from "@unfour/command-client";

export function useDatabaseConnections(workspaceId: string) {
  return useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["database-connections", workspaceId],
    queryFn: () => listDatabaseConnections(workspaceId),
  });
}
