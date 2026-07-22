// @vitest-environment jsdom
import type { ReactNode } from "react";
import type { WorkspaceEnvironment } from "@unfour/command-client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { I18nProvider } from "@unfour/ui";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ApiClientSidebar } from "./ApiClientSidebar";

vi.mock("@unfour/command-client", () => ({
  setActiveWorkspaceEnvironment: vi.fn(),
  createApiCollection: vi.fn(),
  createApiCollectionFolder: vi.fn(),
  createWorkspaceEnvironment: vi.fn(),
  deleteApiCollection: vi.fn(),
  deleteApiCollectionFolder: vi.fn(),
  deleteWorkspaceEnvironment: vi.fn(),
  deleteApiRequest: vi.fn(),
  duplicateApiRequest: vi.fn(),
  listApiCollections: vi.fn(),
  listApiCollectionFolders: vi.fn(),
  listWorkspaceEnvironments: vi.fn(),
  listApiHistory: vi.fn(),
  listSavedApiRequests: vi.fn(),
  moveApiCollectionFolder: vi.fn(),
  moveApiRequest: vi.fn(),
  renameApiCollection: vi.fn(),
  renameApiCollectionFolder: vi.fn(),
  reorderApiCollectionFolders: vi.fn(),
  reorderApiRequests: vi.fn(),
  updateWorkspaceEnvironmentVariables: vi.fn(),
  updateApiRequest: vi.fn(),
}));

import {
  listApiCollections,
  listApiCollectionFolders,
  listWorkspaceEnvironments,
  listApiHistory,
  listSavedApiRequests,
  setActiveWorkspaceEnvironment,
} from "@unfour/command-client";

const listCollectionsMock = vi.mocked(listApiCollections);
const listFoldersMock = vi.mocked(listApiCollectionFolders);
const listSavedMock = vi.mocked(listSavedApiRequests);
const listHistoryMock = vi.mocked(listApiHistory);
const listEnvironmentsMock = vi.mocked(listWorkspaceEnvironments);

function environment(
  overrides: Partial<WorkspaceEnvironment> = {},
): WorkspaceEnvironment {
  return {
    id: "env-1",
    workspaceId: "ws-1",
    name: "Local",
    sortOrder: 0,
    variables: [],
    isActive: false,
    createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-01-01T00:00:00Z",
    deletedAt: null,
    revision: 1,
    syncStatus: "local",
    remoteId: null,
    ...overrides,
  };
}

function createWrapper() {
  const client = new QueryClient({
    defaultOptions: { mutations: { retry: false }, queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <I18nProvider initialLocale="en">
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      </I18nProvider>
    );
  };
}

function renderSidebar(overrides: Partial<Parameters<typeof ApiClientSidebar>[0]> = {}) {
  const props = {
    environmentPanelActive: false,
    onEditEnvironment: vi.fn(),
    onEditWorkspaceVariables: vi.fn(),
    onNewEnvironment: vi.fn(),
    onNewRequest: vi.fn(),
    onOpenEnvironments: vi.fn(),
    onOpenIntent: vi.fn(),
    selectedEnvironmentId: "env-1",
    selectedId: null,
    workspaceId: "ws-1",
    ...overrides,
  };
  render(<ApiClientSidebar {...props} />, { wrapper: createWrapper() });
  return props;
}

beforeEach(() => {
  vi.clearAllMocks();
  listCollectionsMock.mockResolvedValue([]);
  listFoldersMock.mockResolvedValue([]);
  listSavedMock.mockResolvedValue([]);
  listHistoryMock.mockResolvedValue([]);
  listEnvironmentsMock.mockResolvedValue([
    environment({ id: "env-1", name: "Local", isActive: true }),
    environment({ id: "env-2", name: "Staging" }),
  ]);
});

afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

describe("ApiClientSidebar environment panel", () => {
  it("shows environment list in the sidebar when the Environments top button is selected", async () => {
    const props = renderSidebar();

    fireEvent.click(screen.getByRole("button", { name: "Environments" }));

    expect(props.onOpenEnvironments).toHaveBeenCalledTimes(1);
    expect(await screen.findByRole("button", { name: "Local" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Staging" })).toBeInTheDocument();
  });


  it("does not show per-environment actions in the sidebar environment list", async () => {
    renderSidebar({ environmentPanelActive: true });

    expect(await screen.findByRole("button", { name: "Local" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Environment actions" })).toBeNull();
    expect(screen.queryByRole("menuitem", { name: "Delete environment" })).toBeNull();
  });
  it("uses the sidebar New button to open a new environment editor", async () => {
    const props = renderSidebar({ environmentPanelActive: true });

    fireEvent.click(await screen.findByRole("button", { name: "New" }));

    await waitFor(() => expect(props.onNewEnvironment).toHaveBeenCalledTimes(1));
  });

  it("selects an environment for editing without changing the active environment", async () => {
    const props = renderSidebar({ environmentPanelActive: true });

    fireEvent.click(await screen.findByRole("button", { name: "Staging" }));

    expect(props.onEditEnvironment).toHaveBeenCalledWith("env-2");
    expect(vi.mocked(setActiveWorkspaceEnvironment)).not.toHaveBeenCalled();
  });
});
