// @vitest-environment jsdom
import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import type { Workspace } from "@unfour/command-client";
import { WorkspaceMenu } from "./WorkspaceMenu";

afterEach(cleanup);

function workspace(name: string): Workspace {
  return {
    id: `ws-${name}`,
    name,
    isDefault: false,
    lastOpenedAt: null,
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
    deletedAt: null,
    revision: 1,
    syncStatus: "local",
    remoteId: null,
  };
}

function createWrapper() {
  const client = new QueryClient({
    defaultOptions: { mutations: { retry: false }, queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };
}

describe("WorkspaceMenu", () => {
  it("keeps the trigger width fixed while workspace names change", () => {
    const first = workspace("Default Workspace");
    const second = workspace("A much longer workspace name");
    const { rerender } = render(
      <WorkspaceMenu
        activeWorkspace={first}
        onActivateWorkspace={vi.fn()}
        workspaces={[first, second]}
      />,
      { wrapper: createWrapper() },
    );

    const firstTrigger = screen.getByRole("button", { name: /default workspace/i });
    expect(firstTrigger).toHaveClass("w-[180px]");
    expect(firstTrigger.querySelector("svg")).toHaveClass("ml-auto");

    rerender(
      <WorkspaceMenu
        activeWorkspace={second}
        onActivateWorkspace={vi.fn()}
        workspaces={[first, second]}
      />,
    );

    const secondTrigger = screen.getByRole("button", {
      name: /a much longer workspace name/i,
    });
    expect(secondTrigger).toHaveClass("w-[180px]");
    expect(secondTrigger.querySelector("svg")).toHaveClass("ml-auto");
  });
});
