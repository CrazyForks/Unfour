import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  listWorkspaceVariables,
  replaceWorkspaceVariables,
  type WorkspaceVariableInput,
} from "@unfour/command-client";

export function useWorkspaceVariables(workspaceId: string) {
  const queryClient = useQueryClient();
  const query = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["workspace-variables", workspaceId],
    queryFn: () => listWorkspaceVariables(workspaceId),
  });
  const replaceMut = useMutation({
    mutationFn: (variables: WorkspaceVariableInput[]) =>
      replaceWorkspaceVariables(workspaceId, variables),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: ["workspace-variables", workspaceId],
      }),
  });

  return {
    isLoading: query.isLoading,
    replaceMut,
    variables: query.data ?? [],
  };
}
