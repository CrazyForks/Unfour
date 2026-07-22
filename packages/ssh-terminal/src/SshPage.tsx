import { useCallback, useEffect, useState } from "react";
import type { ReactNode } from "react";
import { SshConnectionsPage } from "./TerminalPage";
import { SshTasksPage } from "./components/SshTasksPage";
import { useSshConnections } from "./hooks/useSshConnections";

export function TerminalPage({
  onShellSidebarChange,
  workspaceId,
}: {
  onShellSidebarChange?: (sidebar: ReactNode | null) => void;
  workspaceId: string;
}) {
  const [activeMode, setActiveMode] = useState<"connections" | "tasks">("connections");
  const [connectionsSidebar, setConnectionsSidebar] = useState<ReactNode | null>(null);
  const [tasksSidebar, setTasksSidebar] = useState<ReactNode | null>(null);
  const connectionsQuery = useSshConnections(workspaceId);
  const openConnections = useCallback(() => setActiveMode("connections"), []);
  const openTasks = useCallback(() => setActiveMode("tasks"), []);

  useEffect(() => {
    if (!onShellSidebarChange) return;
    onShellSidebarChange(
      activeMode === "connections" ? connectionsSidebar : tasksSidebar,
    );
  }, [activeMode, connectionsSidebar, onShellSidebarChange, tasksSidebar]);

  useEffect(() => {
    return () => onShellSidebarChange?.(null);
  }, [onShellSidebarChange]);

  return (
    <div className="flex h-full min-h-0 min-w-0 flex-col bg-[var(--u-color-surface)]">
      <div
        className={
          activeMode === "connections"
            ? "flex min-h-0 min-w-0 flex-1 flex-col"
            : "hidden"
        }
      >
        <SshConnectionsPage
          onOpenTasks={openTasks}
          onShellSidebarChange={setConnectionsSidebar}
          workspaceId={workspaceId}
        />
      </div>
      <div
        className={
          activeMode === "tasks" ? "flex min-h-0 min-w-0 flex-1 flex-col" : "hidden"
        }
      >
        <SshTasksPage
          active={activeMode === "tasks"}
          connections={connectionsQuery.data ?? []}
          key={workspaceId}
          onOpenConnections={openConnections}
          onShellSidebarChange={setTasksSidebar}
          workspaceId={workspaceId}
        />
      </div>
    </div>
  );
}
