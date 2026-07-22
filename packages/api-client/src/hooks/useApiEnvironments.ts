import { useMemo } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createWorkspaceEnvironment,
  deleteWorkspaceEnvironment,
  listWorkspaceEnvironments,
  setActiveWorkspaceEnvironment,
  updateWorkspaceEnvironmentVariables,
  type WorkspaceEnvironment,
  type WorkspaceVariableInput,
} from "@unfour/command-client";
import { useFeedbackErrorHandler } from "@unfour/ui";

/**
 * Workspace environment CRUD and local active-environment selection.
 */
export function useApiEnvironments(workspaceId: string) {
  const queryClient = useQueryClient();
  const handleError = useFeedbackErrorHandler();

  const query = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["workspace-environments", workspaceId],
    queryFn: () => listWorkspaceEnvironments(workspaceId),
  });

  const invalidate = () =>
    queryClient.invalidateQueries({ queryKey: ["workspace-environments", workspaceId] });

  const createMut = useMutation({
    mutationFn: (name: string) => createWorkspaceEnvironment(workspaceId, name),
    onSuccess: invalidate,
  });
  const updateMut = useMutation({
    mutationFn: (input: {
      id: string;
      name: string;
      variables: WorkspaceVariableInput[];
    }) =>
      updateWorkspaceEnvironmentVariables(
        workspaceId,
        input.id,
        input.name,
        input.variables,
      ),
    onSuccess: invalidate,
  });
  const deleteMut = useMutation({
    mutationFn: (environmentId: string) =>
      deleteWorkspaceEnvironment(workspaceId, environmentId),
    onSuccess: invalidate,
    onError: (error) =>
      handleError(error, { key: "feedback.api.environmentDeleteFailed" }),
  });
  const activateMut = useMutation({
    mutationFn: (environmentId: string | null) =>
      setActiveWorkspaceEnvironment(workspaceId, environmentId),
    onSuccess: invalidate,
    onError: (error) =>
      handleError(error, { key: "feedback.api.environmentActivateFailed" }),
  });

  const environments = useMemo<WorkspaceEnvironment[]>(
    () => query.data ?? [],
    [query.data],
  );
  const activeEnvironment = useMemo(
    () => environments.find((environment) => environment.isActive) ?? null,
    [environments],
  );

  return {
    activateMut,
    activeEnvironment,
    createMut,
    deleteMut,
    environments,
    isLoading: query.isLoading,
    updateMut,
  };
}
