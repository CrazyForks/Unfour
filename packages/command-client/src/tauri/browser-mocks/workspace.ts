import { getMockLayout, mockState, mockStore, mockWorkspace } from "./state";
import { UNHANDLED, type MockResult } from "./types";
import type {
  Workspace,
  WorkspaceEnvironmentType,
  WorkspaceLayout,
  WorkspaceMcpPolicy,
} from "../../types";

export function handleWorkspaceMock<T>(
  command: string,
  args?: Record<string, unknown>,
): MockResult<T> {
  if (command === "workspace_list") {
    return mockState as T;
  }

  if (command === "workspace_create") {
    const environmentType = String(args?.environmentType ?? "dev") as WorkspaceEnvironmentType;
    const mcpPolicy = String(args?.mcpPolicy ?? "auto") as WorkspaceMcpPolicy;
    const workspace: Workspace = {
      ...mockWorkspace,
      id: crypto.randomUUID(),
      name: String(args?.name ?? "New Workspace"),
      isDefault: false,
      environmentType,
      mcpPolicy,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    mockState.workspaces = [workspace, ...mockState.workspaces];
    mockState.activeWorkspaceId = workspace.id;
    return workspace as T;
  }

  if (command === "workspace_update_environment") {
    const workspaceId = String(args?.workspaceId ?? "");
    const workspace = mockState.workspaces.find((item) => item.id === workspaceId);
    if (!workspace) throw new Error("workspace not found");
    workspace.environmentType = String(
      args?.environmentType ?? workspace.environmentType,
    ) as WorkspaceEnvironmentType;
    workspace.updatedAt = new Date().toISOString();
    return workspace as T;
  }

  if (command === "workspace_rename") {
    const workspaceId = String(args?.workspaceId ?? "");
    const workspace = mockState.workspaces.find((item) => item.id === workspaceId);
    if (!workspace) throw new Error("workspace not found");
    workspace.name = String(args?.name ?? workspace.name);
    workspace.updatedAt = new Date().toISOString();
    return workspace as T;
  }

  if (command === "workspace_delete") {
    const workspaceId = String(args?.workspaceId ?? "");
    if (mockState.workspaces.length <= 1) {
      throw new Error("at least one workspace must remain");
    }
    mockState.workspaces = mockState.workspaces.filter((item) => item.id !== workspaceId);
    if (mockState.activeWorkspaceId === workspaceId) {
      mockState.activeWorkspaceId = mockState.workspaces[0]?.id ?? mockWorkspace.id;
    }
    return mockState as T;
  }

  if (command === "workspace_set_active") {
    mockState.activeWorkspaceId = String(args?.workspaceId ?? mockState.activeWorkspaceId);
    return mockState as T;
  }

  if (command === "workspace_layout_get") {
    const workspaceId = String(args?.workspaceId ?? mockState.activeWorkspaceId);
    return getMockLayout(workspaceId) as T;
  }

  if (command === "workspace_layout_update") {
    const workspaceId = String(args?.workspaceId ?? mockState.activeWorkspaceId);
    const layout = args?.layout as WorkspaceLayout;
    mockStore.layouts[workspaceId] = {
      ...layout,
      workspaceId,
      updatedAt: new Date().toISOString(),
    };
    return mockStore.layouts[workspaceId] as T;
  }

  return UNHANDLED;
}
