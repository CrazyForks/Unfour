import { call } from "./invoke";
import type {
  Workspace,
  WorkspaceEnvironmentType,
  WorkspaceLayout,
  WorkspaceMcpPolicy,
  WorkspaceState,
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
