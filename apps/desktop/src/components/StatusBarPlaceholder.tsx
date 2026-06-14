import { StatusBar } from "@unfour/ui";
import type { Workspace, WorkspaceTab } from "@unfour/command-client";
import { moduleLabel } from "./module-helpers";

export function StatusBarPlaceholder({
  activeTab,
  activeWorkspace,
  healthReady,
  syncStrategy,
}: {
  activeTab: WorkspaceTab;
  activeWorkspace?: Workspace;
  healthReady: boolean;
  syncStrategy: string;
}) {
  return (
    <StatusBar>
      <div className="flex min-w-0 items-center gap-3">
        <span className="truncate">{activeWorkspace?.name ?? "No workspace"}</span>
        <span>{moduleLabel(activeTab)}</span>
      </div>
      <div className="flex shrink-0 items-center gap-3">
        <span>{healthReady ? "Storage ready" : "Checking storage"}</span>
        <span>{syncStrategy}</span>
      </div>
    </StatusBar>
  );
}
