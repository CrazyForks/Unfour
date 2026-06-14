import type { WorkspaceTab } from "@unfour/command-client";

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
