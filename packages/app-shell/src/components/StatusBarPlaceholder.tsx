import type { ReactNode } from "react";
import { StatusBar, useI18n } from "@unfour/ui";
import type { Workspace, WorkspaceTab } from "@unfour/command-client";
import { Bell, CheckCircle2, Circle, GitBranch, Wifi } from "lucide-react";
import { moduleLabel } from "./module-helpers";

export function StatusBarPlaceholder({
  activeTab,
  activeWorkspace,
  healthReady,
  rightAccessory,
  syncStrategy,
}: {
  activeTab: WorkspaceTab;
  activeWorkspace?: Workspace;
  healthReady: boolean;
  rightAccessory?: ReactNode;
  syncStrategy: string;
}) {
  const { t } = useI18n();

  return (
    <StatusBar>
      <div className="flex min-w-0 items-center gap-4">
        <span className="flex min-w-0 items-center gap-1.5">
          {healthReady ? (
            <CheckCircle2 className="shrink-0" size={14} />
          ) : (
            <Circle className="shrink-0 opacity-80" size={13} />
          )}
          <span className="truncate">
            {healthReady ? t("app.status.ready") : t("app.status.checkingStorage")}
          </span>
        </span>
        <span className="truncate opacity-90">
          {activeWorkspace?.name ?? t("app.workspace.none")}
        </span>
        <span className="opacity-90">{moduleLabel(activeTab, t)}</span>
      </div>
      <div className="flex shrink-0 items-center gap-3">
        <span className="hidden items-center gap-1.5 opacity-90 lg:flex">
          <GitBranch size={13} />
          main
        </span>
        <span className="hidden items-center gap-1.5 md:flex">
          <Wifi size={13} />
          {t("app.status.connected")}
        </span>
        <span className="hidden opacity-90 xl:inline">{syncStrategy}</span>
        <span className="hidden font-mono opacity-90 lg:inline">UTF-8</span>
        <span className="hidden font-mono opacity-90 md:inline">
          {activeTab.kind === "api" ? "JSON" : moduleLabel(activeTab, t)}
        </span>
        <Bell className="hidden opacity-90 sm:block" size={13} />
        {rightAccessory}
      </div>
    </StatusBar>
  );
}
