import { create } from "zustand";
import type { WorkspaceTab } from "../types";

type WorkspaceStore = {
  activeWorkspaceId?: string;
  activeTabId: string;
  sidebarCollapsed: boolean;
  tabs: WorkspaceTab[];
  openTab: (tab: WorkspaceTab) => void;
  setActiveTab: (tabId: string) => void;
  setActiveWorkspace: (workspaceId: string) => void;
  toggleSidebar: () => void;
};

const initialTabs: WorkspaceTab[] = [
  { id: "api-main", title: "API Client", kind: "api" },
  { id: "ssh-main", title: "SSH Terminal", kind: "ssh" },
  { id: "database-main", title: "Database", kind: "database" },
];

export const useWorkspaceStore = create<WorkspaceStore>((set) => ({
  activeTabId: "api-main",
  sidebarCollapsed: false,
  tabs: initialTabs,
  openTab: (tab) =>
    set((state) => ({
      tabs: state.tabs.some((item) => item.id === tab.id)
        ? state.tabs
        : [...state.tabs, tab],
      activeTabId: tab.id,
    })),
  setActiveTab: (tabId) => set({ activeTabId: tabId }),
  setActiveWorkspace: (workspaceId) => set({ activeWorkspaceId: workspaceId }),
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
}));
