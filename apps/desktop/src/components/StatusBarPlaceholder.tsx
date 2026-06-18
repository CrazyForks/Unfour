import { StatusBar, useI18n } from "@unfour/ui";
import type { Workspace, WorkspaceTab } from "@unfour/command-client";
import { Bell, CheckCircle2, Circle, GitBranch, Wifi } from "lucide-react";
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
  const { t } = useI18n();

  return (
    <StatusBar>
      <div className="flex min-w-0 items-center gap-4">
        <span className="flex min-w-0 items-center gap-1.5">
          {healthReady ? (
            <CheckCircle2 className="shrink-0 text-[var(--u-color-success)]" size={14} />
          ) : (
            <Circle className="shrink-0 text-[var(--u-color-warning)]" size={13} />
          )}
          <span className="truncate">
            {healthReady ? t("app.status.ready") : t("app.status.checkingStorage")}
          </span>
        </span>
        <span className="truncate">{activeWorkspace?.name ?? t("app.workspace.none")}</span>
        <span>{moduleLabel(activeTab, t)}</span>
      </div>
      <div className="flex shrink-0 items-center gap-4">
        <span className="flex items-center gap-1.5">
          <GitBranch size={13} />
          main
        </span>
        <span className="flex items-center gap-1.5">
          <Wifi className="text-[var(--u-color-success)]" size={13} />
          {t("app.status.connected")}
        </span>
        <span>{syncStrategy}</span>
        <span className="font-mono">UTF-8</span>
        <span className="font-mono">
          {activeTab.kind === "api" ? "JSON" : moduleLabel(activeTab, t)}
        </span>
        <Bell size={13} />
      </div>
    </StatusBar>
  );
}
