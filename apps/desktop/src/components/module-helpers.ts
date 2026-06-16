import type { WorkspaceTab } from "@unfour/command-client";

export type ModuleSwitcherItem = {
  id: "api-main" | "ssh-main" | "database-main";
  kind: WorkspaceTab["kind"];
  label: string;
  shortLabel: string;
};

export function getModuleSwitcherItems(): ModuleSwitcherItem[] {
  return [
    { id: "api-main", kind: "api", label: "API Client", shortLabel: "API" },
    { id: "ssh-main", kind: "ssh", label: "SSH Terminal", shortLabel: "SSH" },
    { id: "database-main", kind: "database", label: "Database", shortLabel: "DB" },
  ];
}

export function moduleLabel(tab: WorkspaceTab) {
  if (tab.kind === "api") {
    return "API";
  }
  if (tab.kind === "ssh") {
    return "SSH";
  }
  return "Database";
}

export function moduleSubtitle(tab: WorkspaceTab) {
  if (tab.kind === "api") {
    return "Collections and requests";
  }
  if (tab.kind === "ssh") {
    return "Connections and sessions";
  }
  return "Connections and schemas";
}

export function isTauriRuntime() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}
