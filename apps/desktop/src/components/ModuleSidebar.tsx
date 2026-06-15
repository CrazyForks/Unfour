import {
  Database,
  Globe2,
  PanelLeftClose,
  PanelLeftOpen,
  TerminalSquare,
} from "lucide-react";
import {
  ApiCollectionTree,
  type ApiOpenIntent,
} from "@unfour/api-client";
import {
  DatabaseConnectionTree,
} from "@unfour/database";
import {
  SshConnectionTree,
} from "@unfour/ssh-terminal";
import type {
  DatabaseConnection,
  WorkspaceTab,
} from "@unfour/command-client";
import {
  IconButton,
  Sidebar,
  SidebarHeader,
  SidebarRow,
  SidebarSection,
} from "@unfour/ui";
import { moduleLabel, moduleSubtitle } from "./module-helpers";

export function ModuleSidebar({
  activeTab,
  activeTabId,
  activeWorkspaceId,
  collapsed,
  databaseConnections,
  onSelectApiRequest,
  onOpenApiIntent,
  onSelectDatabaseConnection,
  onToggle,
  selectedApiRequestId,
  selectedDatabaseConnectionId,
  setActiveTab,
  setSelectedApiRequest,
}: {
  activeTab: WorkspaceTab;
  activeTabId: string;
  activeWorkspaceId: string;
  collapsed: boolean;
  databaseConnections: DatabaseConnection[];
  onSelectApiRequest: (requestId: string) => void;
  onOpenApiIntent: (intent: ApiOpenIntent) => void;
  onSelectDatabaseConnection: (connection: DatabaseConnection) => void;
  onToggle: () => void;
  selectedApiRequestId: string | null;
  selectedDatabaseConnectionId: string | null;
  setActiveTab: (tabId: string) => void;
  setSelectedApiRequest: (requestId: string | null) => void;
}) {
  return (
    <Sidebar
      collapsed={collapsed}
      header={
        <SidebarHeader>
          <div className="flex h-[26px] w-[26px] items-center justify-center rounded-[var(--u-radius-sm)] border border-[var(--u-color-border)] bg-[var(--u-color-surface)] text-[var(--u-color-text-muted)]">
            {activeTab.kind === "api" && <Globe2 size={15} />}
            {activeTab.kind === "ssh" && <TerminalSquare size={15} />}
            {activeTab.kind === "database" && <Database size={15} />}
          </div>
          {!collapsed && (
            <div className="min-w-0 flex-1">
              <div className="truncate text-[13px] font-semibold text-[var(--u-color-text)]">
                {moduleLabel(activeTab)}
              </div>
              <div className="truncate text-[12px] text-[var(--u-color-text-muted)]">
                {moduleSubtitle(activeTab)}
              </div>
            </div>
          )}
          <IconButton className="ml-auto" label="Toggle sidebar" onClick={onToggle}>
            {collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
          </IconButton>
        </SidebarHeader>
      }
    >

      {activeTab.kind === "api" && (
        <ApiCollectionTree
          active={activeTabId === "api-main"}
          collapsed={collapsed}
          onOpenClient={() => {
            setSelectedApiRequest(null);
            onOpenApiIntent({ kind: "new", nonce: Date.now() });
            setActiveTab("api-main");
          }}
          onOpenIntent={(intent) => {
            if (intent.kind === "saved") {
              onSelectApiRequest(intent.requestId);
            }
            onOpenApiIntent(intent);
          }}
          selectedId={selectedApiRequestId}
          workspaceId={activeWorkspaceId}
        />
      )}
      {activeTab.kind === "ssh" && (
        <SshConnectionTree
          active={activeTabId === "ssh-main"}
          collapsed={collapsed}
          onOpenTerminal={() => setActiveTab("ssh-main")}
          workspaceId={activeWorkspaceId}
        />
      )}
      {activeTab.kind === "database" && (
        <div className="space-y-3">
          <ResourceGroup collapsed={collapsed} title="Database">
            <SidebarAction
              collapsed={collapsed}
              icon={<Database size={14} />}
              label="SQL Workspace"
              onClick={() => setActiveTab("database-main")}
              selected={activeTabId === "database-main" && (collapsed || !selectedDatabaseConnectionId)}
            />
            {!collapsed && (
              <DatabaseConnectionTree
                connections={databaseConnections}
                onNewQuery={() => setActiveTab("database-main")}
                onSelectConnection={onSelectDatabaseConnection}
                selectedConnectionId={selectedDatabaseConnectionId}
              />
            )}
          </ResourceGroup>
        </div>
      )}
    </Sidebar>
  );
}

function ResourceGroup({
  children,
  collapsed,
  title,
}: {
  children: React.ReactNode;
  collapsed: boolean;
  title: string;
}) {
  return (
    <SidebarSection title={collapsed ? undefined : title}>
      <div className="space-y-1">{children}</div>
    </SidebarSection>
  );
}

function SidebarAction({
  collapsed,
  icon,
  label,
  onClick,
  selected,
}: {
  collapsed: boolean;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  selected: boolean;
}) {
  return (
    <SidebarRow active={selected} onClick={onClick}>
      {icon}
      {!collapsed && <span className="truncate">{label}</span>}
    </SidebarRow>
  );
}
