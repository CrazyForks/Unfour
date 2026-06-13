import { useQuery } from "@tanstack/react-query";
import { getDatabaseSchema } from "@unfour/command-client";
import type { DatabaseConnection } from "@unfour/command-client";

export function useSchemaTree({
  connection,
  connectionId,
  workspaceId,
}: {
  connection: DatabaseConnection | null;
  connectionId: string | null;
  workspaceId: string;
}) {
  return useQuery({
    enabled: Boolean(workspaceId && connectionId && connection),
    queryKey: ["database-schema", workspaceId, connectionId],
    queryFn: () => getDatabaseSchema(workspaceId, connectionId ?? ""),
  });
}
