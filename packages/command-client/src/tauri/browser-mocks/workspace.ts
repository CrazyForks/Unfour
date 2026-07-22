import { getMockLayout, mockState, mockStore, mockWorkspace } from "./state";
import { UNHANDLED, type MockResult } from "./types";
import type {
  Workspace,
  WorkspaceEnvironment,
  WorkspaceEnvironmentVariable,
  WorkspaceEnvironmentType,
  WorkspaceLayout,
  WorkspaceMcpPolicy,
  WorkspaceVariable,
  WorkspaceVariableInput,
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

  if (command === "workspace_variables_list") {
    const workspaceId = String(args?.workspaceId ?? "");
    return mockStore.workspaceVariables
      .filter((variable) => variable.workspaceId === workspaceId && !variable.deletedAt)
      .sort((left, right) => left.sortOrder - right.sortOrder) as T;
  }

  if (command === "workspace_variables_replace") {
    const workspaceId = String(args?.workspaceId ?? "");
    const inputs = (args?.variables ?? []) as WorkspaceVariableInput[];
    const current = mockStore.workspaceVariables.filter(
      (variable) => variable.workspaceId !== workspaceId,
    );
    const now = new Date().toISOString();
    const next = inputs.map((input, index): WorkspaceVariable => {
      const existing = mockStore.workspaceVariables.find(
        (variable) => variable.workspaceId === workspaceId && variable.id === input.id,
      );
      return {
        ...input,
        id: existing?.id ?? crypto.randomUUID(),
        workspaceId,
        sortOrder: index,
        createdAt: existing?.createdAt ?? now,
        updatedAt: now,
        deletedAt: null,
        revision: (existing?.revision ?? 0) + 1,
        syncStatus: "local",
        remoteId: existing?.remoteId ?? null,
      };
    });
    mockStore.workspaceVariables = [...current, ...next];
    return next as T;
  }

  if (command === "workspace_environments_list") {
    const workspaceId = String(args?.workspaceId ?? "");
    return workspaceEnvironmentList(workspaceId) as T;
  }

  if (command === "workspace_environment_create") {
    const workspaceId = String(args?.workspaceId ?? "");
    const now = new Date().toISOString();
    const existing = workspaceEnvironmentList(workspaceId);
    const environment: WorkspaceEnvironment = {
      id: crypto.randomUUID(),
      workspaceId,
      name: String(args?.name ?? "New Environment"),
      sortOrder: existing.length,
      // Mirror backend: the first environment becomes active.
      isActive: existing.length === 0,
      variables: [],
      createdAt: now,
      updatedAt: now,
      deletedAt: null,
      revision: 1,
      syncStatus: "local",
      remoteId: null,
    };
    mockStore.workspaceEnvironments.push(environment);
    return environment as T;
  }

  if (command === "workspace_environment_update") {
    const workspaceId = String(args?.workspaceId ?? "");
    const environmentId = String(args?.environmentId ?? "");
    const environment = mockStore.workspaceEnvironments.find(
      (item) => item.id === environmentId && item.workspaceId === workspaceId,
    );
    if (!environment) throw new Error("workspace environment not found");
    const inputs = (args?.variables ?? []) as WorkspaceVariableInput[];
    const now = new Date().toISOString();
    environment.name = String(args?.name ?? environment.name);
    environment.updatedAt = now;
    environment.revision += 1;
    environment.variables = inputs.map((input, index): WorkspaceEnvironmentVariable => {
      const existing = environment.variables.find((variable) => variable.id === input.id);
      return {
        ...input,
        id: existing?.id ?? crypto.randomUUID(),
        workspaceId,
        environmentId,
        sortOrder: index,
        createdAt: existing?.createdAt ?? now,
        updatedAt: now,
        deletedAt: null,
        revision: (existing?.revision ?? 0) + 1,
        syncStatus: "local",
        remoteId: existing?.remoteId ?? null,
      };
    });
    return environment as T;
  }

  if (command === "workspace_environment_delete") {
    const workspaceId = String(args?.workspaceId ?? "");
    const environmentId = String(args?.environmentId ?? "");
    const environment = mockStore.workspaceEnvironments.find(
      (item) => item.id === environmentId && item.workspaceId === workspaceId,
    );
    if (!environment) throw new Error("workspace environment not found");
    const wasActive = environment.isActive;
    mockStore.workspaceEnvironments = mockStore.workspaceEnvironments.filter(
      (item) => item !== environment,
    );
    const remaining = workspaceEnvironmentList(workspaceId);
    if (wasActive && remaining[0]) remaining[0].isActive = true;
    return remaining as T;
  }

  if (command === "workspace_environment_set_active") {
    const workspaceId = String(args?.workspaceId ?? "");
    const environmentId = args?.environmentId == null ? null : String(args.environmentId);
    if (
      environmentId &&
      !mockStore.workspaceEnvironments.some(
        (environment) =>
          environment.id === environmentId && environment.workspaceId === workspaceId,
      )
    ) {
      throw new Error("workspace environment not found");
    }
    for (const environment of mockStore.workspaceEnvironments) {
      if (environment.workspaceId === workspaceId) {
        environment.isActive = environment.id === environmentId;
      }
    }
    return workspaceEnvironmentList(workspaceId) as T;
  }

  if (command === "workspace_variables_resolve") {
    const workspaceId = String(args?.workspaceId ?? "");
    const environmentId =
      args?.activeEnvironmentId == null ? null : String(args.activeEnvironmentId);
    const environment = environmentId
      ? mockStore.workspaceEnvironments.find(
          (item) => item.id === environmentId && item.workspaceId === workspaceId,
        )
      : null;
    if (environmentId && !environment) throw new Error("workspace environment not found");
    const values = new Map(
      mockStore.workspaceVariables
        .filter((variable) => variable.workspaceId === workspaceId && variable.isEnabled)
        .map((variable) => [variable.key, variable.value]),
    );
    for (const variable of environment?.variables ?? []) {
      if (variable.isEnabled) values.set(variable.key, variable.value);
    }
    const input = String(args?.input ?? "");
    return input.replace(/\{\{\s*([^{}]+?)\s*\}\}/g, (_token, key: string) => {
      const value = values.get(key);
      if (value === undefined) throw new Error(`unresolved variable: ${key}`);
      return value;
    }) as T;
  }

  return UNHANDLED;
}

function workspaceEnvironmentList(workspaceId: string) {
  return mockStore.workspaceEnvironments
    .filter((environment) => environment.workspaceId === workspaceId && !environment.deletedAt)
    .sort((left, right) => left.sortOrder - right.sortOrder);
}
