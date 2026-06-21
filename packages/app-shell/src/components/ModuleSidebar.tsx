import type { ReactNode } from "react";
import type { WorkspaceTab } from "@unfour/command-client";
import { Sidebar } from "@unfour/ui";

export function ModuleSidebar({
  activeTab,
  apiSidebarContent,
  collapsed,
  databaseSidebarContent,
  onWidthChange,
  sshSidebarContent,
  width,
}: {
  activeTab: WorkspaceTab;
  apiSidebarContent?: ReactNode;
  collapsed: boolean;
  databaseSidebarContent?: ReactNode;
  onWidthChange: (width: number) => void;
  sshSidebarContent?: ReactNode;
  width: number;
}) {
  if (collapsed) {
    return null;
  }

  return (
    <Sidebar
      contentClassName={activeTab.kind === "api" ? "overflow-hidden p-0" : undefined}
      onWidthChange={onWidthChange}
      resizable
      width={width}
    >
      {activeTab.kind === "api" && apiSidebarContent}
      {activeTab.kind === "ssh" && sshSidebarContent}
      {activeTab.kind === "database" && databaseSidebarContent}
    </Sidebar>
  );
}
