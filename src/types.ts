export type Workspace = {
  id: string;
  name: string;
  isDefault: boolean;
  lastOpenedAt: string | null;
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

export type KeyValue = {
  key: string;
  value: string;
  enabled: boolean;
};

export type WorkspaceEnvironment = {
  workspaceId: string;
  variables: KeyValue[];
  updatedAt: string;
};

export type ApiRequestInput = {
  workspaceId: string;
  name?: string;
  method: string;
  url: string;
  headers: KeyValue[];
  query: KeyValue[];
  body?: string;
  bodyKind: string;
  timeoutMs?: number;
};

export type ApiResponse = {
  historyId: string;
  status: number;
  statusText: string;
  headers: KeyValue[];
  body: string;
  durationMs: number;
};

export type ApiHistoryItem = {
  id: string;
  workspaceId: string;
  name: string | null;
  method: string;
  url: string;
  status: number | null;
  durationMs: number | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  revision: number;
  syncStatus: string;
  remoteId: string | null;
};

export type ApiSavedRequest = {
  id: string;
  workspaceId: string;
  name: string;
  method: string;
  url: string;
  headersJson: string;
  queryJson: string;
  body: string | null;
  bodyKind: string;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  revision: number;
  syncStatus: string;
  remoteId: string | null;
};

export type SystemHealth = {
  appName: string;
  storageReady: boolean;
  commandBusReady: boolean;
  aiReservedCapabilities: string[];
  syncStrategy: string;
};

export type WorkspaceTab = {
  id: string;
  title: string;
  kind: "api" | "ssh" | "database";
};
