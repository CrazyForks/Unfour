import { describe, expect, it, beforeEach } from "vitest";
import { useWorkspaceStore } from "./workspace-store";

beforeEach(() => {
  useWorkspaceStore.setState({
    activeTabId: "api-main",
    selectedApiRequestId: null,
    selectedDatabaseConnectionId: null,
    selectedSshConnectionId: null,
    sidebarCollapsed: false,
    tabs: [
      { id: "api-main", title: "API Client", kind: "api" },
      { id: "ssh-main", title: "SSH Terminal", kind: "ssh" },
      { id: "database-main", title: "Database", kind: "database" },
    ],
    activeWorkspaceId: undefined,
    layoutWorkspaceId: undefined,
  });
});

describe("workspace store initial state", () => {
  it("has correct defaults", () => {
    const state = useWorkspaceStore.getState();
    expect(state.activeTabId).toBe("api-main");
    expect(state.selectedApiRequestId).toBeNull();
    expect(state.selectedDatabaseConnectionId).toBeNull();
    expect(state.selectedSshConnectionId).toBeNull();
    expect(state.sidebarCollapsed).toBe(false);
    expect(state.tabs).toHaveLength(3);
  });
});

describe("setActiveTab", () => {
  it("updates activeTabId", () => {
    useWorkspaceStore.getState().setActiveTab("ssh-main");
    expect(useWorkspaceStore.getState().activeTabId).toBe("ssh-main");
  });
});

describe("setActiveWorkspace", () => {
  it("updates activeWorkspaceId", () => {
    useWorkspaceStore.getState().setActiveWorkspace("ws-1");
    expect(useWorkspaceStore.getState().activeWorkspaceId).toBe("ws-1");
  });
});

describe("toggleSidebar", () => {
  it("toggles sidebarCollapsed", () => {
    expect(useWorkspaceStore.getState().sidebarCollapsed).toBe(false);
    useWorkspaceStore.getState().toggleSidebar();
    expect(useWorkspaceStore.getState().sidebarCollapsed).toBe(true);
    useWorkspaceStore.getState().toggleSidebar();
    expect(useWorkspaceStore.getState().sidebarCollapsed).toBe(false);
  });
});

describe("selection setters", () => {
  it("sets selectedApiRequestId", () => {
    useWorkspaceStore.getState().setSelectedApiRequest("req-1");
    expect(useWorkspaceStore.getState().selectedApiRequestId).toBe("req-1");
    useWorkspaceStore.getState().setSelectedApiRequest(null);
    expect(useWorkspaceStore.getState().selectedApiRequestId).toBeNull();
  });

  it("sets selectedDatabaseConnectionId", () => {
    useWorkspaceStore.getState().setSelectedDatabaseConnection("db-1");
    expect(useWorkspaceStore.getState().selectedDatabaseConnectionId).toBe("db-1");
  });

  it("sets selectedSshConnectionId", () => {
    useWorkspaceStore.getState().setSelectedSshConnection("ssh-1");
    expect(useWorkspaceStore.getState().selectedSshConnectionId).toBe("ssh-1");
  });
});

describe("openTab", () => {
  it("adds a new tab and activates it", () => {
    const newTab = { id: "custom-1", title: "Custom", kind: "api" as const };
    useWorkspaceStore.getState().openTab(newTab);
    const state = useWorkspaceStore.getState();
    expect(state.tabs).toHaveLength(4);
    expect(state.activeTabId).toBe("custom-1");
  });

  it("does not duplicate existing tabs", () => {
    useWorkspaceStore.getState().openTab({ id: "api-main", title: "API Client", kind: "api" });
    expect(useWorkspaceStore.getState().tabs).toHaveLength(3);
    expect(useWorkspaceStore.getState().activeTabId).toBe("api-main");
  });
});

describe("hydrateLayout", () => {
  it("restores layout state", () => {
    useWorkspaceStore.getState().hydrateLayout({
      workspaceId: "ws-hydrated",
      activeTabId: "database-main",
      sidebarCollapsed: true,
      tabs: [
        { id: "database-main", title: "Database", kind: "database" },
      ],
      selectedApiRequestId: "req-h",
      selectedDatabaseConnectionId: "db-h",
      selectedSshConnectionId: null,
      updatedAt: "2026-01-01T00:00:00Z",
    });
    const state = useWorkspaceStore.getState();
    expect(state.activeTabId).toBe("database-main");
    expect(state.sidebarCollapsed).toBe(true);
    expect(state.layoutWorkspaceId).toBe("ws-hydrated");
    expect(state.selectedApiRequestId).toBe("req-h");
    expect(state.selectedDatabaseConnectionId).toBe("db-h");
    expect(state.tabs).toHaveLength(1);
  });

  it("uses default tabs when layout tabs are empty", () => {
    useWorkspaceStore.getState().hydrateLayout({
      workspaceId: "ws-empty",
      activeTabId: "api-main",
      sidebarCollapsed: false,
      tabs: [],
      selectedApiRequestId: null,
      selectedDatabaseConnectionId: null,
      selectedSshConnectionId: null,
      updatedAt: "2026-01-01T00:00:00Z",
    });
    expect(useWorkspaceStore.getState().tabs).toHaveLength(3);
  });
});

describe("snapshotLayout", () => {
  it("captures current state as a layout snapshot", () => {
    useWorkspaceStore.setState({
      activeTabId: "ssh-main",
      sidebarCollapsed: true,
      selectedApiRequestId: "req-snap",
      selectedDatabaseConnectionId: null,
      selectedSshConnectionId: "ssh-conn-1",
      tabs: [
        { id: "api-main", title: "API Client", kind: "api" },
        { id: "ssh-main", title: "SSH Terminal", kind: "ssh" },
      ],
    });
    const snapshot = useWorkspaceStore.getState().snapshotLayout("ws-snap");
    expect(snapshot.workspaceId).toBe("ws-snap");
    expect(snapshot.activeTabId).toBe("ssh-main");
    expect(snapshot.sidebarCollapsed).toBe(true);
    expect(snapshot.selectedApiRequestId).toBe("req-snap");
    expect(snapshot.selectedSshConnectionId).toBe("ssh-conn-1");
    expect(snapshot.tabs).toHaveLength(2);
    expect(snapshot.updatedAt).toBeDefined();
  });
});
