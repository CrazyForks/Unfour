import { call } from "./invoke";
import type {
  Workspace,
  WorkspaceEnvironment,
  WorkspaceEnvironmentType,
  WorkspaceLayout,
  WorkspaceMcpPolicy,
  WorkspaceState,
  WorkspaceVariable,
  WorkspaceVariableInput,
} from "../types";

export function getWorkspaceState() {
  return call<WorkspaceState>("workspace_list");
}

export function createWorkspace(
  name: string,
  environmentType?: WorkspaceEnvironmentType,
  mcpPolicy?: WorkspaceMcpPolicy,
) {
  return call<Workspace>("workspace_create", { name, environmentType, mcpPolicy });
}

export function renameWorkspace(workspaceId: string, name: string) {
  return call<Workspace>("workspace_rename", { workspaceId, name });
}

export function deleteWorkspace(workspaceId: string) {
  return call<WorkspaceState>("workspace_delete", { workspaceId });
}

export function setActiveWorkspace(workspaceId: string) {
  return call<WorkspaceState>("workspace_set_active", { workspaceId });
}

export function updateWorkspaceEnvironment(
  workspaceId: string,
  environmentType: WorkspaceEnvironmentType,
) {
  return call<Workspace>("workspace_update_environment", {
    workspaceId,
    environmentType,
  });
}

export function getWorkspaceLayout(workspaceId: string) {
  return call<WorkspaceLayout>("workspace_layout_get", { workspaceId });
}

export function updateWorkspaceLayout(workspaceId: string, layout: WorkspaceLayout) {
  return call<WorkspaceLayout>("workspace_layout_update", {
    workspaceId,
    layout,
  });
}

export function listWorkspaceVariables(workspaceId: string) {
  return call<WorkspaceVariable[]>("workspace_variables_list", { workspaceId });
}

export function replaceWorkspaceVariables(
  workspaceId: string,
  variables: WorkspaceVariableInput[],
) {
  return call<WorkspaceVariable[]>("workspace_variables_replace", {
    workspaceId,
    variables,
  });
}

export function listWorkspaceEnvironments(workspaceId: string) {
  return call<WorkspaceEnvironment[]>("workspace_environments_list", { workspaceId });
}

export function createWorkspaceEnvironment(workspaceId: string, name: string) {
  return call<WorkspaceEnvironment>("workspace_environment_create", {
    workspaceId,
    name,
  });
}

export function updateWorkspaceEnvironmentVariables(
  workspaceId: string,
  environmentId: string,
  name: string,
  variables: WorkspaceVariableInput[],
) {
  return call<WorkspaceEnvironment>("workspace_environment_update", {
    workspaceId,
    environmentId,
    name,
    variables,
  });
}

export function deleteWorkspaceEnvironment(
  workspaceId: string,
  environmentId: string,
) {
  return call<WorkspaceEnvironment[]>("workspace_environment_delete", {
    workspaceId,
    environmentId,
  });
}

export function setActiveWorkspaceEnvironment(
  workspaceId: string,
  environmentId: string | null,
) {
  return call<WorkspaceEnvironment[]>("workspace_environment_set_active", {
    workspaceId,
    environmentId,
  });
}

export function resolveWorkspaceVariables(
  workspaceId: string,
  activeEnvironmentId: string | null,
  input: string,
) {
  return call<string>("workspace_variables_resolve", {
    workspaceId,
    activeEnvironmentId,
    input,
  });
}
