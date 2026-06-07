import * as Dialog from "@radix-ui/react-dialog";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import {
  Activity,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Database,
  Folder,
  Globe2,
  Home,
  Maximize2,
  Minus,
  MoreHorizontal,
  Pencil,
  PanelLeftClose,
  PanelLeftOpen,
  Plus,
  Search,
  Square,
  TerminalSquare,
  Trash2,
  X,
} from "lucide-react";
import {
  ApiCollectionTree,
  ApiDebuggerPage,
} from "@unfour/api-debugger";
import { AppShell } from "@unfour/app-shell";
import {
  DatabaseConnectionTree,
  DatabasePage,
} from "@unfour/database";
import {
  SshConnectionTree,
  TerminalLogPanel,
  TerminalPage,
  TerminalStatusBar,
} from "@unfour/terminal";
import { FormEvent, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Badge,
  BottomPanel,
  Button,
  CommandPalette,
  GlobalToolbar,
  IconButton,
  Input,
  MainWorkspace,
  RightInspector,
  Sidebar,
  SidebarHeader,
  SidebarRow,
  SidebarSection,
  StatusBar,
  TabBar,
  cn,
} from "@unfour/ui";
import {
  createWorkspace,
  deleteWorkspace,
  getSystemHealth,
  getWorkspaceLayout,
  getWorkspaceState,
  listDatabaseConnections,
  renameWorkspace,
  setActiveWorkspace as setActiveWorkspaceCommand,
  updateWorkspaceLayout,
} from "@unfour/command-client";
import { useWorkspaceStore } from "@unfour/workspace";
import type {
  DatabaseConnection,
  Workspace,
  WorkspaceTab,
} from "@unfour/command-client";

function App() {
  const queryClient = useQueryClient();
  const [bottomPanelCollapsed, setBottomPanelCollapsed] = useState(true);
  const [bottomPanelHeight, setBottomPanelHeight] = useState(220);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [rightInspectorCollapsed, setRightInspectorCollapsed] = useState(true);
  const [rightInspectorWidth] = useState(300);
  const {
    activeTabId,
    activeWorkspaceId,
    hydrateLayout,
    layoutWorkspaceId,
    selectedApiRequestId,
    selectedDatabaseConnectionId,
    selectedSshConnectionId,
    setActiveTab,
    setActiveWorkspace,
    setSelectedApiRequest,
    setSelectedDatabaseConnection,
    sidebarCollapsed,
    snapshotLayout,
    toggleSidebar,
    tabs,
  } = useWorkspaceStore();
  const healthQuery = useQuery({
    queryKey: ["system-health"],
    queryFn: getSystemHealth,
  });
  const workspaceQuery = useQuery({
    queryKey: ["workspaces"],
    queryFn: getWorkspaceState,
  });

  const activeWorkspace =
    workspaceQuery.data?.workspaces.find(
      (workspace) => workspace.id === (activeWorkspaceId || workspaceQuery.data.activeWorkspaceId),
    ) ?? workspaceQuery.data?.workspaces[0];

  const workspaceLayoutQuery = useQuery({
    enabled: Boolean(activeWorkspace?.id),
    queryKey: ["workspace-layout", activeWorkspace?.id],
    queryFn: () => getWorkspaceLayout(activeWorkspace?.id ?? ""),
  });
  const sidebarDatabaseConnectionsQuery = useQuery({
    enabled: Boolean(activeWorkspace?.id),
    queryKey: ["database-connections", activeWorkspace?.id],
    queryFn: () => listDatabaseConnections(activeWorkspace?.id ?? ""),
  });

  useEffect(() => {
    if (workspaceQuery.data?.activeWorkspaceId && !activeWorkspaceId) {
      setActiveWorkspace(workspaceQuery.data.activeWorkspaceId);
    }
  }, [activeWorkspaceId, setActiveWorkspace, workspaceQuery.data?.activeWorkspaceId]);

  useEffect(() => {
    if (workspaceLayoutQuery.data) {
      hydrateLayout(workspaceLayoutQuery.data);
    }
  }, [hydrateLayout, workspaceLayoutQuery.data]);

  useEffect(() => {
    const items = sidebarDatabaseConnectionsQuery.data;
    if (
      selectedDatabaseConnectionId &&
      items &&
      !items.some((item) => item.id === selectedDatabaseConnectionId)
    ) {
      setSelectedDatabaseConnection(null);
    }
  }, [
    selectedDatabaseConnectionId,
    setSelectedDatabaseConnection,
    sidebarDatabaseConnectionsQuery.data,
  ]);

  const layoutMutation = useMutation({
    mutationFn: (workspaceId: string) =>
      updateWorkspaceLayout(workspaceId, snapshotLayout(workspaceId)),
  });

  useEffect(() => {
    if (!activeWorkspace?.id || layoutWorkspaceId !== activeWorkspace.id) {
      return;
    }

    const timeout = window.setTimeout(() => {
      layoutMutation.mutate(activeWorkspace.id);
    }, 350);

    return () => window.clearTimeout(timeout);
  }, [
    activeTabId,
    activeWorkspace?.id,
    layoutWorkspaceId,
    selectedApiRequestId,
    selectedDatabaseConnectionId,
    selectedSshConnectionId,
    sidebarCollapsed,
    tabs,
  ]);

  const createWorkspaceMutation = useMutation({
    mutationFn: createWorkspace,
    onSuccess: (workspace) => {
      setActiveWorkspace(workspace.id);
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });

  const activateWorkspaceMutation = useMutation({
    mutationFn: setActiveWorkspaceCommand,
    onSuccess: (state) => {
      setActiveWorkspace(state.activeWorkspaceId);
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });

  const renameWorkspaceMutation = useMutation({
    mutationFn: ({ name, workspaceId }: { name: string; workspaceId: string }) =>
      renameWorkspace(workspaceId, name),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["workspaces"] }),
  });

  const deleteWorkspaceMutation = useMutation({
    mutationFn: deleteWorkspace,
    onSuccess: (state) => {
      setActiveWorkspace(state.activeWorkspaceId);
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });

  const activeTab = tabs.find((tab) => tab.id === activeTabId) ?? tabs[0];

  return (
    <>
      <AppShell
        bottomPanel={
          activeTab.kind === "ssh" && activeWorkspace ? (
            <TerminalLogPanel
              collapsed={bottomPanelCollapsed}
              height={bottomPanelHeight}
              onCollapse={() => setBottomPanelCollapsed(true)}
              onHeightChange={setBottomPanelHeight}
              workspaceId={activeWorkspace.id}
            />
          ) : (
          <BottomPanel
            collapsed={bottomPanelCollapsed}
            height={bottomPanelHeight}
            onHeightChange={setBottomPanelHeight}
            resizable
          >
            <div className="flex h-[var(--u-size-section-toolbar)] items-center justify-between border-b border-[var(--u-color-border)] px-2">
              <div className="flex items-center gap-2 text-[12px] font-semibold text-[var(--u-color-text)]">
                <Activity size={14} />
                Diagnostics
              </div>
              <IconButton label="Collapse bottom panel" onClick={() => setBottomPanelCollapsed(true)}>
                <Minus size={14} />
              </IconButton>
            </div>
            <div className="p-2 text-[12px] text-[var(--u-color-text-muted)]">
              Local activity and module diagnostics will appear here.
            </div>
          </BottomPanel>
          )
        }
        globalToolbar={
          <AppTitleBar
            activeWorkspace={activeWorkspace}
            createWorkspaceMutation={createWorkspaceMutation}
            deleteWorkspaceMutation={deleteWorkspaceMutation}
            healthReady={healthQuery.data?.storageReady === true}
            onActivateWorkspace={(workspaceId) => activateWorkspaceMutation.mutate(workspaceId)}
            onOpenCommandPalette={() => setCommandPaletteOpen(true)}
            onToggleBottomPanel={() => setBottomPanelCollapsed((current) => !current)}
            onToggleInspector={() => setRightInspectorCollapsed((current) => !current)}
            renameWorkspaceMutation={renameWorkspaceMutation}
            syncStrategy={healthQuery.data?.syncStrategy ?? "local-first"}
            workspaces={workspaceQuery.data?.workspaces ?? []}
          />
        }
        rightInspector={
          <RightInspector collapsed={rightInspectorCollapsed} width={rightInspectorWidth}>
            <div className="flex h-[var(--u-size-section-toolbar)] items-center justify-between border-b border-[var(--u-color-border)] px-2">
              <div className="text-[12px] font-semibold text-[var(--u-color-text)]">
                Inspector
              </div>
              <IconButton label="Collapse inspector" onClick={() => setRightInspectorCollapsed(true)}>
                <PanelLeftOpen size={14} />
              </IconButton>
            </div>
            <div className="p-2 text-[12px] text-[var(--u-color-text-muted)]">
              {moduleLabel(activeTab)} details and properties will use this space.
            </div>
          </RightInspector>
        }
        sidebar={
          <ModuleSidebar
            activeTab={activeTab}
            activeTabId={activeTabId}
            activeWorkspaceId={activeWorkspace?.id ?? ""}
            collapsed={sidebarCollapsed}
            databaseConnections={sidebarDatabaseConnectionsQuery.data ?? []}
            onSelectApiRequest={(requestId) => {
              setSelectedApiRequest(requestId);
              setActiveTab("api-main");
            }}
            onSelectDatabaseConnection={(connection) => {
              setSelectedDatabaseConnection(connection.id);
              setActiveTab("database-main");
            }}
            onToggle={toggleSidebar}
            selectedApiRequestId={selectedApiRequestId}
            selectedDatabaseConnectionId={selectedDatabaseConnectionId}
            setActiveTab={setActiveTab}
            setSelectedApiRequest={setSelectedApiRequest}
          />
        }
        statusBar={
          activeTab.kind === "ssh" && activeWorkspace ? (
            <TerminalStatusBar
              workspaceId={activeWorkspace.id}
              workspaceName={activeWorkspace.name}
            />
          ) : (
          <StatusBar>
            <div className="flex min-w-0 items-center gap-3">
              <span className="truncate">{activeWorkspace?.name ?? "No workspace"}</span>
              <span>{moduleLabel(activeTab)}</span>
            </div>
            <div className="flex shrink-0 items-center gap-3">
              <span>{healthQuery.data?.storageReady === true ? "Storage ready" : "Checking storage"}</span>
              <span>{healthQuery.data?.syncStrategy ?? "local-first"}</span>
            </div>
          </StatusBar>
          )
        }
        main={
        <MainWorkspace
          tabBar={
            <TabBar
              activeTabId={activeTabId}
              onSelectTab={setActiveTab}
              tabs={tabs.map((tab) => ({
                id: tab.id,
                meta: tab.kind,
                title: moduleLabel(tab),
              }))}
            />
          }
        >
          {activeTab.kind === "api" && activeWorkspace && (
            <ApiDebuggerPage
              selectedRequestId={selectedApiRequestId}
              setSelectedRequestId={setSelectedApiRequest}
              workspaceId={activeWorkspace.id}
            />
          )}
          {activeTab.kind === "ssh" && activeWorkspace && (
            <TerminalPage workspaceId={activeWorkspace.id} />
          )}
          {activeTab.kind === "database" && activeWorkspace && (
            <DatabasePage workspaceId={activeWorkspace.id} />
          )}
        </MainWorkspace>
        }
      />
      <CommandPalette
        actions={
          <>
            <CommandPaletteAction onSelect={() => setActiveTab("api-main")}>
              Open API Debugger
            </CommandPaletteAction>
            <CommandPaletteAction onSelect={() => setActiveTab("database-main")}>
              Open Database
            </CommandPaletteAction>
            <CommandPaletteAction onSelect={() => setActiveTab("ssh-main")}>
              Open SSH Terminal
            </CommandPaletteAction>
          </>
        }
        onClose={() => setCommandPaletteOpen(false)}
        open={commandPaletteOpen}
      />
    </>
  );
}

function CommandPaletteAction({
  children,
  onSelect,
}: {
  children: React.ReactNode;
  onSelect: () => void;
}) {
  return (
    <button
      className="flex h-[var(--u-size-sidebar-row)] w-full items-center rounded-[var(--u-radius-sm)] px-2 text-left text-[13px] text-[var(--u-color-text-muted)] hover:bg-[var(--u-color-surface-hover)] hover:text-[var(--u-color-text)]"
      onClick={onSelect}
      type="button"
    >
      {children}
    </button>
  );
}

type PendingMutation<TVariables> = {
  isPending: boolean;
  mutate: (variables: TVariables) => void;
};

function AppTitleBar({
  activeWorkspace,
  createWorkspaceMutation,
  deleteWorkspaceMutation,
  healthReady,
  onActivateWorkspace,
  onOpenCommandPalette,
  onToggleBottomPanel,
  onToggleInspector,
  renameWorkspaceMutation,
  syncStrategy,
  workspaces,
}: {
  activeWorkspace?: Workspace;
  createWorkspaceMutation: PendingMutation<string>;
  deleteWorkspaceMutation: PendingMutation<string>;
  healthReady: boolean;
  onActivateWorkspace: (workspaceId: string) => void;
  onOpenCommandPalette: () => void;
  onToggleBottomPanel: () => void;
  onToggleInspector: () => void;
  renameWorkspaceMutation: PendingMutation<{ name: string; workspaceId: string }>;
  syncStrategy: string;
  workspaces: Workspace[];
}) {
  async function dragWindow(event: React.MouseEvent<HTMLDivElement>) {
    if (event.button !== 0 || !isTauriRuntime()) {
      return;
    }

    await getCurrentWindow().startDragging();
  }

  return (
    <GlobalToolbar
      center={
        <button
          className="flex h-[var(--u-size-input)] w-full max-w-[520px] items-center gap-2 rounded-[var(--u-radius-sm)] border border-[var(--u-color-border)] bg-[var(--u-color-surface-subtle)] px-2 text-[12px] text-[var(--u-color-text-muted)] transition-colors hover:bg-[var(--u-color-surface-hover)]"
          onClick={onOpenCommandPalette}
          onMouseDown={(event) => event.stopPropagation()}
          type="button"
        >
          <Search size={15} />
          <span className="truncate">Search or run command</span>
        </button>
      }
      left={
        <>
          <IconButton disabled label="Back">
            <ChevronLeft size={16} />
          </IconButton>
          <IconButton disabled label="Forward">
            <ChevronRight size={16} />
          </IconButton>
          <IconButton label="Home">
            <Home size={16} />
          </IconButton>
          <div className="mx-1 h-5 w-px bg-[var(--u-color-border)]" />
          <div className="flex h-[26px] w-[26px] items-center justify-center rounded-[var(--u-radius-sm)] bg-[var(--u-color-primary)] text-[var(--u-color-primary-foreground)]">
            <Activity size={15} />
          </div>
          <WorkspaceMenu
            activeWorkspace={activeWorkspace}
            createWorkspaceMutation={createWorkspaceMutation}
            deleteWorkspaceMutation={deleteWorkspaceMutation}
            onActivateWorkspace={onActivateWorkspace}
            renameWorkspaceMutation={renameWorkspaceMutation}
            workspaces={workspaces}
          />
        </>
      }
      onDragRegionMouseDown={dragWindow}
      right={
        <>
          <Badge tone={healthReady ? "green" : "amber"}>
            {healthReady ? "local storage" : "checking"}
          </Badge>
          <Badge tone="neutral">{syncStrategy}</Badge>
          <IconButton label="Toggle inspector" onClick={onToggleInspector}>
            <PanelLeftClose size={16} />
          </IconButton>
          <IconButton label="Toggle bottom panel" onClick={onToggleBottomPanel}>
            <Maximize2 size={15} />
          </IconButton>
          <IconButton label="More actions">
            <MoreHorizontal size={16} />
          </IconButton>
          <WindowControls />
        </>
      }
    />
  );
}

function WorkspaceMenu({
  activeWorkspace,
  createWorkspaceMutation,
  deleteWorkspaceMutation,
  onActivateWorkspace,
  renameWorkspaceMutation,
  workspaces,
}: {
  activeWorkspace?: Workspace;
  createWorkspaceMutation: PendingMutation<string>;
  deleteWorkspaceMutation: PendingMutation<string>;
  onActivateWorkspace: (workspaceId: string) => void;
  renameWorkspaceMutation: PendingMutation<{ name: string; workspaceId: string }>;
  workspaces: Workspace[];
}) {
  const [createOpen, setCreateOpen] = useState(false);
  const [renameOpen, setRenameOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [workspaceName, setWorkspaceName] = useState("");
  const [renameDraft, setRenameDraft] = useState(activeWorkspace?.name ?? "");
  const canDelete =
    Boolean(activeWorkspace) && !activeWorkspace?.isDefault && workspaces.length > 1;

  useEffect(() => {
    setRenameDraft(activeWorkspace?.name ?? "");
  }, [activeWorkspace?.id, activeWorkspace?.name]);

  function createWorkspaceFromDialog(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const name = workspaceName.trim();
    if (!name) {
      return;
    }
    createWorkspaceMutation.mutate(name);
    setWorkspaceName("");
    setCreateOpen(false);
  }

  function renameWorkspaceFromDialog(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const name = renameDraft.trim();
    if (!activeWorkspace || !name || name === activeWorkspace.name) {
      return;
    }
    renameWorkspaceMutation.mutate({ workspaceId: activeWorkspace.id, name });
    setRenameOpen(false);
  }

  function deleteWorkspaceFromDialog() {
    if (!activeWorkspace || !canDelete) {
      return;
    }
    deleteWorkspaceMutation.mutate(activeWorkspace.id);
    setDeleteOpen(false);
  }

  return (
    <>
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <Button
            className="ml-2 max-w-[240px] justify-start gap-1 border-transparent bg-white px-2 font-semibold shadow-none hover:bg-slate-100"
            size="sm"
            type="button"
            variant="outline"
          >
            <span className="min-w-0 truncate">
              {activeWorkspace?.name ?? "No workspace"}
            </span>
            <ChevronDown className="shrink-0 text-slate-500" size={14} />
          </Button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            align="start"
            className="z-50 w-72 rounded-md border border-slate-200 bg-white p-1 text-sm text-slate-800 shadow-xl"
            sideOffset={6}
          >
            <DropdownMenu.Label className="px-2 py-1.5 text-xs font-semibold uppercase text-slate-500">
              Workspaces
            </DropdownMenu.Label>
            {workspaces.map((workspace) => (
              <DropdownMenu.Item
                className={cn(
                  "flex min-h-8 cursor-pointer items-center gap-2 rounded px-2 py-1.5 outline-none hover:bg-slate-100 focus:bg-slate-100",
                  activeWorkspace?.id === workspace.id && "bg-teal-50 text-teal-900",
                )}
                key={workspace.id}
                onSelect={() => onActivateWorkspace(workspace.id)}
              >
                <Folder size={14} />
                <span className="min-w-0 flex-1 truncate">{workspace.name}</span>
                {workspace.isDefault && <Badge tone="teal">default</Badge>}
              </DropdownMenu.Item>
            ))}
            {workspaces.length === 0 && (
              <div className="px-2 py-4 text-center text-xs text-slate-500">
                No workspaces
              </div>
            )}
            <DropdownMenu.Separator className="my-1 h-px bg-slate-200" />
            <DropdownMenu.Item
              className="flex h-8 cursor-pointer items-center gap-2 rounded px-2 outline-none hover:bg-slate-100 focus:bg-slate-100"
              onSelect={() => setCreateOpen(true)}
            >
              <Plus size={14} />
              New workspace
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="flex h-8 cursor-pointer items-center gap-2 rounded px-2 outline-none hover:bg-slate-100 focus:bg-slate-100 disabled:pointer-events-none disabled:opacity-50"
              disabled={!activeWorkspace}
              onSelect={() => setRenameOpen(true)}
            >
              <Pencil size={14} />
              Rename current
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="flex h-8 cursor-pointer items-center gap-2 rounded px-2 text-rose-700 outline-none hover:bg-rose-50 focus:bg-rose-50 disabled:pointer-events-none disabled:opacity-50"
              disabled={!canDelete}
              onSelect={() => setDeleteOpen(true)}
            >
              <Trash2 size={14} />
              Delete current
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>

      <WorkspaceDialog
        description="Create a workspace for a separate set of API requests, SSH connections, and database resources."
        disabled={createWorkspaceMutation.isPending || !workspaceName.trim()}
        onOpenChange={setCreateOpen}
        onSubmit={createWorkspaceFromDialog}
        open={createOpen}
        setValue={setWorkspaceName}
        submitLabel="Create"
        title="New workspace"
        value={workspaceName}
      />

      <WorkspaceDialog
        description="Rename the active workspace. Existing workspace-scoped records stay attached to it."
        disabled={
          renameWorkspaceMutation.isPending ||
          !renameDraft.trim() ||
          renameDraft.trim() === activeWorkspace?.name
        }
        onOpenChange={setRenameOpen}
        onSubmit={renameWorkspaceFromDialog}
        open={renameOpen}
        setValue={setRenameDraft}
        submitLabel="Rename"
        title="Rename workspace"
        value={renameDraft}
      />

      <Dialog.Root onOpenChange={setDeleteOpen} open={deleteOpen}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 z-50 bg-slate-950/30" />
          <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-[360px] -translate-x-1/2 -translate-y-1/2 rounded-md border border-slate-200 bg-white p-4 shadow-xl">
            <Dialog.Title className="text-base font-semibold text-slate-950">
              Delete workspace
            </Dialog.Title>
            <Dialog.Description className="mt-2 text-sm text-slate-600">
              Delete {activeWorkspace?.name ?? "this workspace"} locally. The app will switch
              to another available workspace.
            </Dialog.Description>
            <div className="mt-4 flex justify-end gap-2">
              <Dialog.Close asChild>
                <Button type="button" variant="outline">
                  Cancel
                </Button>
              </Dialog.Close>
              <Button
                className="bg-rose-700 hover:bg-rose-800"
                disabled={deleteWorkspaceMutation.isPending || !canDelete}
                onClick={deleteWorkspaceFromDialog}
                type="button"
              >
                <Trash2 size={15} />
                Delete
              </Button>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </>
  );
}

function WorkspaceDialog({
  description,
  disabled,
  onOpenChange,
  onSubmit,
  open,
  setValue,
  submitLabel,
  title,
  value,
}: {
  description: string;
  disabled: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setValue: (value: string) => void;
  submitLabel: string;
  title: string;
  value: string;
}) {
  return (
    <Dialog.Root onOpenChange={onOpenChange} open={open}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-slate-950/30" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-[360px] -translate-x-1/2 -translate-y-1/2 rounded-md border border-slate-200 bg-white p-4 shadow-xl">
          <Dialog.Title className="text-base font-semibold text-slate-950">
            {title}
          </Dialog.Title>
          <Dialog.Description className="mt-2 text-sm text-slate-600">
            {description}
          </Dialog.Description>
          <form className="mt-4 space-y-4" onSubmit={onSubmit}>
            <Input
              autoFocus
              onChange={(event) => setValue(event.target.value)}
              placeholder="Workspace name"
              value={value}
            />
            <div className="flex justify-end gap-2">
              <Dialog.Close asChild>
                <Button type="button" variant="outline">
                  Cancel
                </Button>
              </Dialog.Close>
              <Button disabled={disabled} type="submit">
                {submitLabel}
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function ModuleSidebar({
  activeTab,
  activeTabId,
  activeWorkspaceId,
  collapsed,
  databaseConnections,
  onSelectApiRequest,
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
            setActiveTab("api-main");
          }}
          onSelectRequest={onSelectApiRequest}
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

function WindowControls() {
  if (!isTauriRuntime()) {
    return (
      <div className="ml-1 flex items-center gap-1 text-slate-300">
        <Minus size={15} />
        <Square size={13} />
        <X size={15} />
      </div>
    );
  }

  const appWindow = getCurrentWindow();

  return (
    <div className="ml-1 flex items-center">
      <TitlebarWindowButton
        ariaLabel="Minimize"
        icon={<Minus size={16} />}
        onClick={() => void appWindow.minimize()}
      />
      <TitlebarWindowButton
        ariaLabel="Maximize"
        icon={<Maximize2 size={14} />}
        onClick={() => void appWindow.toggleMaximize()}
      />
      <TitlebarWindowButton
        ariaLabel="Close"
        className="hover:bg-rose-600 hover:text-white"
        icon={<X size={16} />}
        onClick={() => void appWindow.close()}
      />
    </div>
  );
}

function TitlebarWindowButton({
  ariaLabel,
  className,
  icon,
  onClick,
}: {
  ariaLabel: string;
  className?: string;
  icon: React.ReactNode;
  onClick: () => void;
}) {
  return (
    <button
      aria-label={ariaLabel}
      className={cn(
        "flex h-8 w-10 items-center justify-center rounded-sm text-slate-600 transition-colors hover:bg-slate-100 hover:text-slate-950",
        className,
      )}
      onClick={onClick}
      type="button"
    >
      {icon}
    </button>
  );
}

function moduleLabel(tab: WorkspaceTab) {
  if (tab.kind === "api") {
    return "API";
  }
  if (tab.kind === "ssh") {
    return "SSH";
  }
  return "Database";
}

function moduleSubtitle(tab: WorkspaceTab) {
  if (tab.kind === "api") {
    return "Collections and requests";
  }
  if (tab.kind === "ssh") {
    return "Connections and sessions";
  }
  return "Connections and schemas";
}

function isTauriRuntime() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
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

export default App;
