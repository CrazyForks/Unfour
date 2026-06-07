import { useEffect } from "react";
import { useMutation } from "@tanstack/react-query";
import { updateWorkspaceLayout } from "@unfour/command-client";
import { useWorkspaceStore } from "@unfour/workspace";

export function useLayoutPersistence(activeWorkspaceId: string | null) {
  const {
    activeTabId,
    layoutWorkspaceId,
    selectedApiRequestId,
    selectedDatabaseConnectionId,
    selectedSshConnectionId,
    sidebarCollapsed,
    snapshotLayout,
    tabs,
  } = useWorkspaceStore();

  const layoutMutation = useMutation({
    mutationFn: (workspaceId: string) =>
      updateWorkspaceLayout(workspaceId, snapshotLayout(workspaceId)),
  });

  useEffect(() => {
    if (!activeWorkspaceId || layoutWorkspaceId !== activeWorkspaceId) {
      return;
    }

    const timeout = window.setTimeout(() => {
      layoutMutation.mutate(activeWorkspaceId);
    }, 350);

    return () => window.clearTimeout(timeout);
  }, [
    activeTabId,
    activeWorkspaceId,
    layoutWorkspaceId,
    selectedApiRequestId,
    selectedDatabaseConnectionId,
    selectedSshConnectionId,
    sidebarCollapsed,
    tabs,
  ]);

  return { layoutMutation, snapshotLayout };
}
