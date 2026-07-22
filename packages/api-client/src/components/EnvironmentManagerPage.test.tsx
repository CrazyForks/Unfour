// @vitest-environment jsdom
import type { ReactNode } from "react";
import type {
  WorkspaceEnvironment,
  WorkspaceEnvironmentVariable,
  WorkspaceVariable,
} from "@unfour/command-client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { I18nProvider } from "@unfour/ui";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EnvironmentManagerPage } from "./EnvironmentManagerPage";

vi.mock("@unfour/command-client", () => ({
  createWorkspaceEnvironment: vi.fn(),
  deleteWorkspaceEnvironment: vi.fn(),
  listWorkspaceEnvironments: vi.fn(),
  listWorkspaceVariables: vi.fn(),
  replaceWorkspaceVariables: vi.fn(),
  setActiveWorkspaceEnvironment: vi.fn(),
  updateWorkspaceEnvironmentVariables: vi.fn(),
}));

import {
  createWorkspaceEnvironment,
  listWorkspaceEnvironments,
  listWorkspaceVariables,
  replaceWorkspaceVariables,
  updateWorkspaceEnvironmentVariables,
} from "@unfour/command-client";

const listEnvironmentsMock = vi.mocked(listWorkspaceEnvironments);
const listVariablesMock = vi.mocked(listWorkspaceVariables);
const createMock = vi.mocked(createWorkspaceEnvironment);
const updateMock = vi.mocked(updateWorkspaceEnvironmentVariables);
const replaceMock = vi.mocked(replaceWorkspaceVariables);

function environmentVariable(
  overrides: Partial<WorkspaceEnvironmentVariable> = {},
): WorkspaceEnvironmentVariable {
  return {
    id: "var-1",
    workspaceId: "ws-1",
    environmentId: "env-1",
    key: "base_url",
    value: "https://local.example.com",
    isSecret: false,
    isEnabled: true,
    description: null,
    sortOrder: 0,
    createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-01-01T00:00:00Z",
    deletedAt: null,
    revision: 1,
    syncStatus: "local",
    remoteId: null,
    ...overrides,
  };
}

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

function workspaceVariable(
  overrides: Partial<WorkspaceVariable> = {},
): WorkspaceVariable {
  const variable = environmentVariable(overrides);
  const { environmentId: _environmentId, ...workspaceVariable } = variable;
  return workspaceVariable;
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

function renderManager(
  initialMode: React.ComponentProps<typeof EnvironmentManagerPage>["initialMode"] = {
    kind: "manage",
    nonce: 1,
  },
) {
  render(
    <EnvironmentManagerPage initialMode={initialMode} workspaceId="ws-1" />,
    { wrapper: createWrapper() },
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  listEnvironmentsMock.mockResolvedValue([environment()]);
  listVariablesMock.mockResolvedValue([]);
  createMock.mockResolvedValue(environment({ id: "env-2", name: "QA" }));
  updateMock.mockResolvedValue(
    environment({
      id: "env-2",
      name: "QA",
      variables: [
        environmentVariable({
          id: "var-2",
          environmentId: "env-2",
          value: "https://qa.example.com",
        }),
      ],
    }),
  );
  replaceMock.mockResolvedValue([
    workspaceVariable({ key: "base_url", value: "https://workspace.example.com" }),
  ]);
});

afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

describe("EnvironmentManagerPage", () => {
  it("creates a new environment and then saves its variables", async () => {
    listEnvironmentsMock.mockResolvedValue([]);
    renderManager({ kind: "new", nonce: 1 });

    fireEvent.change(await screen.findByLabelText("Name"), { target: { value: "QA" } });
    fireEvent.click(screen.getByRole("button", { name: "Add variable" }));
    fireEvent.change(screen.getByPlaceholderText("Key"), {
      target: { value: "base_url" },
    });
    fireEvent.change(screen.getByPlaceholderText("Value"), {
      target: { value: "https://qa.example.com" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(createMock).toHaveBeenCalledWith("ws-1", "QA"));
    await waitFor(() =>
      expect(updateMock).toHaveBeenCalledWith("ws-1", "env-2", "QA", [
        {
          key: "base_url",
          value: "https://qa.example.com",
          isSecret: false,
          isEnabled: true,
          description: null,
          sortOrder: 0,
        },
      ]),
    );
  });

  it("edits the fixed workspace variable collection", async () => {
    renderManager({ kind: "workspace", nonce: 1 });

    expect(await screen.findByRole("heading", { name: "Workspace Variables" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Add variable" }));
    fireEvent.change(screen.getByPlaceholderText("Key"), { target: { value: "base_url" } });
    fireEvent.change(screen.getByPlaceholderText("Value"), {
      target: { value: "https://workspace.example.com" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(replaceMock).toHaveBeenCalledWith("ws-1", [
        {
          key: "base_url",
          value: "https://workspace.example.com",
          isSecret: false,
          isEnabled: true,
          description: null,
          sortOrder: 0,
        },
      ]),
    );
  });

  it("blocks duplicate environment names before save", async () => {
    renderManager({ kind: "new", nonce: 1 });
    fireEvent.change(await screen.findByLabelText("Name"), {
      target: { value: "local" },
    });

    expect(
      screen.getByText("An environment named local already exists in this workspace."),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });
});
