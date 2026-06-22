// @vitest-environment jsdom
import type { ReactNode } from "react";
import type { ApiEnvironment } from "@unfour/command-client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { I18nProvider } from "@unfour/ui";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EnvironmentControl } from "./EnvironmentControl";

vi.mock("@unfour/command-client", () => ({
  activateApiEnvironment: vi.fn(),
  createApiEnvironment: vi.fn(),
  deleteApiEnvironment: vi.fn(),
  listApiEnvironments: vi.fn(),
  updateApiEnvironment: vi.fn(),
}));

import {
  createApiEnvironment,
  deleteApiEnvironment,
  listApiEnvironments,
  updateApiEnvironment,
} from "@unfour/command-client";

const listMock = vi.mocked(listApiEnvironments);
const createMock = vi.mocked(createApiEnvironment);
const updateMock = vi.mocked(updateApiEnvironment);
const deleteMock = vi.mocked(deleteApiEnvironment);

function environment(overrides: Partial<ApiEnvironment> = {}): ApiEnvironment {
  return {
    id: "env-1",
    workspaceId: "ws-1",
    name: "Local",
    variables: [],
    isActive: false,
    createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-01-01T00:00:00Z",
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

function renderControl(overrides: Partial<Parameters<typeof EnvironmentControl>[0]> = {}) {
  const props = {
    activeEnvironmentId: "env-2",
    onCreateEnvironment: vi.fn(),
    onEditEnvironment: vi.fn(),
    onManageEnvironments: vi.fn(),
    onSelectEnvironment: vi.fn(),
    workspaceId: "ws-1",
    ...overrides,
  };
  render(<EnvironmentControl {...props} />, { wrapper: createWrapper() });
  return props;
}

beforeEach(() => {
  vi.clearAllMocks();
  listMock.mockResolvedValue([
    environment({ id: "env-1", name: "Local" }),
    environment({ id: "env-2", name: "Staging", isActive: true }),
  ]);
});

afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

describe("EnvironmentControl", () => {
  it("switches environments without running environment CRUD", async () => {
    const props = renderControl();

    fireEvent.click(screen.getByRole("button", { name: "Active environment" }));
    fireEvent.click(await screen.findByRole("button", { name: "Local" }));

    expect(props.onSelectEnvironment).toHaveBeenCalledWith("env-1");
    expect(createMock).not.toHaveBeenCalled();
    expect(updateMock).not.toHaveBeenCalled();
    expect(deleteMock).not.toHaveBeenCalled();
  });

  it("uses footer actions to jump to the environment manager", async () => {
    const props = renderControl();

    fireEvent.click(screen.getByRole("button", { name: "Active environment" }));
    fireEvent.click(await screen.findByRole("button", { name: "Manage environments" }));
    expect(props.onManageEnvironments).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByRole("button", { name: "Active environment" }));
    fireEvent.click(await screen.findByRole("button", { name: "New environment" }));
    expect(props.onCreateEnvironment).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByRole("button", { name: "Active environment" }));
    fireEvent.click(await screen.findByRole("button", { name: "Edit current environment" }));
    expect(props.onEditEnvironment).toHaveBeenCalledWith("env-2");

    await waitFor(() => expect(listMock).toHaveBeenCalledWith("ws-1"));
  });
});
