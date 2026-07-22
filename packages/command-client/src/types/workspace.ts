export type WorkspaceEnvironmentType = "dev" | "test" | "prod";

export type WorkspaceMcpPolicy =
  | "auto"
  | "disabled"
  | "read_only"
  | "guarded"
  | "full_access";

export type Workspace = {
  id: string;
  name: string;
  isDefault: boolean;
  lastOpenedAt: string | null;
  environmentType: WorkspaceEnvironmentType;
  mcpPolicy: WorkspaceMcpPolicy;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  revision: number;
  syncStatus: string;
  remoteId: string | null;
};

export type WorkspaceState = {
  activeWorkspaceId: string;
  workspaces: Workspace[];
};

export type WorkspaceVariableInput = {
  id?: string | null;
  key: string;
  value: string;
  isSecret: boolean;
  isEnabled: boolean;
  description: string | null;
  sortOrder: number;
};

export type WorkspaceVariable = WorkspaceVariableInput & {
  id: string;
  workspaceId: string;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  revision: number;
  syncStatus: string;
  remoteId: string | null;
};

export type WorkspaceEnvironmentVariable = WorkspaceVariable & {
  environmentId: string;
};

export type WorkspaceEnvironment = {
  id: string;
  workspaceId: string;
  name: string;
  sortOrder: number;
  isActive: boolean;
  variables: WorkspaceEnvironmentVariable[];
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  revision: number;
  syncStatus: string;
  remoteId: string | null;
};

export type WorkspaceTab = {
  id: string;
  title: string;
  kind: "api" | "ssh" | "database";
};

export type WorkspaceLayout = {
  workspaceId: string;
  sidebarCollapsed: boolean;
  activeTabId: string;
  tabs: WorkspaceTab[];
  selectedApiRequestId: string | null;
  selectedDatabaseConnectionId: string | null;
  selectedSshConnectionId: string | null;
  sidebarWidth?: number;
  bottomPanelHeight?: number;
  rightInspectorWidth?: number;
  updatedAt: string;
};
