import Editor from "@monaco-editor/react";
import {
  Activity,
  Braces,
  ChevronLeft,
  Clock,
  Database,
  Folder,
  Globe2,
  History,
  KeyRound,
  Pencil,
  Plus,
  Save,
  Send,
  Server,
  Settings,
  TerminalSquare,
  Trash2,
} from "lucide-react";
import { FormEvent, useEffect, useMemo, useState } from "react";
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { TerminalPreview } from "./components/TerminalPreview";
import { Badge } from "./components/ui/badge";
import { Button } from "./components/ui/button";
import { Input } from "./components/ui/input";
import {
  createWorkspace,
  getSystemHealth,
  getWorkspaceEnvironment,
  getWorkspaceState,
  deleteWorkspace,
  listApiHistory,
  listSavedApiRequests,
  renameWorkspace,
  saveApiRequest,
  sendApiRequest,
  setActiveWorkspace as setActiveWorkspaceCommand,
  updateWorkspaceEnvironment,
} from "./lib/tauri";
import { cn } from "./lib/utils";
import { useWorkspaceStore } from "./store/workspace-store";
import type {
  ApiHistoryItem,
  ApiRequestInput,
  ApiResponse,
  ApiSavedRequest,
  KeyValue,
} from "./types";

const methods = ["GET", "POST", "PUT", "PATCH", "DELETE"];

function App() {
  const queryClient = useQueryClient();
  const {
    activeTabId,
    activeWorkspaceId,
    setActiveTab,
    setActiveWorkspace,
    sidebarCollapsed,
    toggleSidebar,
    tabs,
  } = useWorkspaceStore();
  const [newWorkspaceName, setNewWorkspaceName] = useState("");
  const [workspaceDraftName, setWorkspaceDraftName] = useState("");

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

  useEffect(() => {
    if (workspaceQuery.data?.activeWorkspaceId && !activeWorkspaceId) {
      setActiveWorkspace(workspaceQuery.data.activeWorkspaceId);
    }
  }, [activeWorkspaceId, setActiveWorkspace, workspaceQuery.data?.activeWorkspaceId]);

  useEffect(() => {
    if (activeWorkspace?.name) {
      setWorkspaceDraftName(activeWorkspace.name);
    }
  }, [activeWorkspace?.id, activeWorkspace?.name]);

  const createWorkspaceMutation = useMutation({
    mutationFn: createWorkspace,
    onSuccess: (workspace) => {
      setNewWorkspaceName("");
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
    <div className="flex h-screen min-h-[680px] bg-zinc-100 text-zinc-950">
      <aside
        className={cn(
          "flex h-full shrink-0 flex-col border-r border-zinc-200 bg-white transition-all",
          sidebarCollapsed ? "w-[64px]" : "w-[292px]",
        )}
      >
        <div className="flex h-14 items-center gap-2 border-b border-zinc-200 px-3">
          <div className="flex h-8 w-8 items-center justify-center rounded-md bg-zinc-950 text-white">
            <Activity size={17} />
          </div>
          {!sidebarCollapsed && (
            <div className="min-w-0">
              <div className="truncate text-sm font-semibold">Unfour Workspace</div>
              <div className="truncate text-xs text-zinc-500">
                {healthQuery.data?.syncStrategy ?? "local-first"}
              </div>
            </div>
          )}
          <Button
            aria-label="Toggle sidebar"
            className="ml-auto"
            onClick={toggleSidebar}
            size="icon"
            type="button"
            variant="ghost"
          >
            <ChevronLeft
              className={cn("transition-transform", sidebarCollapsed && "rotate-180")}
              size={17}
            />
          </Button>
        </div>

        <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto p-3">
          {!sidebarCollapsed && (
            <form
              className="flex gap-2"
              onSubmit={(event) => {
                event.preventDefault();
                if (newWorkspaceName.trim()) {
                  createWorkspaceMutation.mutate(newWorkspaceName);
                }
              }}
            >
              <Input
                onChange={(event) => setNewWorkspaceName(event.target.value)}
                placeholder="Workspace name"
                value={newWorkspaceName}
              />
              <Button disabled={createWorkspaceMutation.isPending} size="icon" type="submit">
                <Plus size={16} />
              </Button>
            </form>
          )}

          <ResourceGroup
            collapsed={sidebarCollapsed}
            icon={<Folder size={16} />}
            title="Workspaces"
          >
            {workspaceQuery.data?.workspaces.map((workspace) => (
              <button
                className={cn(
                  "flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-sm",
                  activeWorkspace?.id === workspace.id
                    ? "bg-teal-50 text-teal-800"
                    : "text-zinc-700 hover:bg-zinc-100",
                )}
                key={workspace.id}
                onClick={() => activateWorkspaceMutation.mutate(workspace.id)}
                type="button"
              >
                <Folder size={14} />
                {!sidebarCollapsed && (
                  <>
                    <span className="min-w-0 flex-1 truncate">{workspace.name}</span>
                    {workspace.isDefault && <Badge tone="teal">default</Badge>}
                  </>
                )}
              </button>
            ))}
          </ResourceGroup>

          <ResourceGroup
            collapsed={sidebarCollapsed}
            icon={<Globe2 size={16} />}
            title="API Collections"
          >
            <SidebarAction
              collapsed={sidebarCollapsed}
              icon={<Braces size={14} />}
              label="REST Client"
              onClick={() => setActiveTab("api-main")}
              selected={activeTabId === "api-main"}
            />
          </ResourceGroup>

          <ResourceGroup
            collapsed={sidebarCollapsed}
            icon={<Server size={16} />}
            title="Connections"
          >
            <SidebarAction
              collapsed={sidebarCollapsed}
              icon={<TerminalSquare size={14} />}
              label="SSH Sessions"
              onClick={() => setActiveTab("ssh-main")}
              selected={activeTabId === "ssh-main"}
            />
            <SidebarAction
              collapsed={sidebarCollapsed}
              icon={<Database size={14} />}
              label="Databases"
              onClick={() => setActiveTab("database-main")}
              selected={activeTabId === "database-main"}
            />
          </ResourceGroup>
        </div>

        <div className="border-t border-zinc-200 p-3">
          <SidebarAction
            collapsed={sidebarCollapsed}
            icon={<Settings size={15} />}
            label="Settings"
            onClick={() => setActiveTab("api-main")}
            selected={false}
          />
        </div>
      </aside>

      <main className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-14 items-center justify-between border-b border-zinc-200 bg-white px-4">
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <Input
                className="h-8 w-[260px] border-transparent bg-zinc-50 font-semibold"
                onChange={(event) => setWorkspaceDraftName(event.target.value)}
                value={workspaceDraftName}
              />
              <Button
                disabled={
                  !activeWorkspace ||
                  renameWorkspaceMutation.isPending ||
                  workspaceDraftName.trim() === activeWorkspace.name
                }
                onClick={() =>
                  activeWorkspace &&
                  renameWorkspaceMutation.mutate({
                    workspaceId: activeWorkspace.id,
                    name: workspaceDraftName,
                  })
                }
                size="icon"
                type="button"
                variant="ghost"
              >
                <Pencil size={15} />
              </Button>
              <Button
                disabled={
                  !activeWorkspace ||
                  activeWorkspace.isDefault ||
                  deleteWorkspaceMutation.isPending ||
                  (workspaceQuery.data?.workspaces.length ?? 0) <= 1
                }
                onClick={() =>
                  activeWorkspace && deleteWorkspaceMutation.mutate(activeWorkspace.id)
                }
                size="icon"
                type="button"
                variant="ghost"
              >
                <Trash2 size={15} />
              </Button>
            </div>
            <div className="flex items-center gap-2 text-xs text-zinc-500">
              <Badge tone={healthQuery.data?.storageReady ? "green" : "amber"}>
                {healthQuery.data?.storageReady ? "local storage" : "checking"}
              </Badge>
              <span>{healthQuery.data?.aiReservedCapabilities.length ?? 0} AI hooks reserved</span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Badge tone="neutral">offline-first</Badge>
            <Badge tone="teal">command bus</Badge>
          </div>
        </header>

        <div className="flex h-10 items-end gap-1 border-b border-zinc-200 bg-white px-3">
          {tabs.map((tab) => (
            <button
              className={cn(
                "flex h-8 items-center gap-2 rounded-t-md border border-b-0 px-3 text-sm",
                activeTabId === tab.id
                  ? "border-zinc-200 bg-zinc-100 text-zinc-950"
                  : "border-transparent text-zinc-500 hover:bg-zinc-50",
              )}
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              type="button"
            >
              {tab.kind === "api" && <Globe2 size={14} />}
              {tab.kind === "ssh" && <TerminalSquare size={14} />}
              {tab.kind === "database" && <Database size={14} />}
              {tab.title}
            </button>
          ))}
        </div>

        <section className="min-h-0 flex-1 overflow-hidden p-4">
          {activeTab.kind === "api" && activeWorkspace && (
            <ApiClientPanel workspaceId={activeWorkspace.id} />
          )}
          {activeTab.kind === "ssh" && <SshPanel />}
          {activeTab.kind === "database" && <DatabasePanel />}
        </section>
      </main>
    </div>
  );
}

function ResourceGroup({
  children,
  collapsed,
  icon,
  title,
}: {
  children: React.ReactNode;
  collapsed: boolean;
  icon: React.ReactNode;
  title: string;
}) {
  return (
    <div>
      <div className="mb-1 flex h-7 items-center gap-2 px-1 text-xs font-semibold uppercase tracking-wide text-zinc-500">
        {icon}
        {!collapsed && title}
      </div>
      <div className="space-y-1">{children}</div>
    </div>
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
    <button
      className={cn(
        "flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-sm",
        selected ? "bg-zinc-950 text-white" : "text-zinc-700 hover:bg-zinc-100",
      )}
      onClick={onClick}
      type="button"
    >
      {icon}
      {!collapsed && <span className="truncate">{label}</span>}
    </button>
  );
}

function ApiClientPanel({ workspaceId }: { workspaceId: string }) {
  const queryClient = useQueryClient();
  const [method, setMethod] = useState("GET");
  const [url, setUrl] = useState("{{base_url}}/get");
  const [name, setName] = useState("Health check");
  const [headers, setHeaders] = useState<KeyValue[]>([
    { key: "Accept", value: "application/json", enabled: true },
  ]);
  const [query, setQuery] = useState<KeyValue[]>([
    { key: "source", value: "{{source}}", enabled: true },
  ]);
  const [body, setBody] = useState("{\n  \"hello\": \"workspace\"\n}");
  const [envVariables, setEnvVariables] = useState<KeyValue[]>([]);
  const [response, setResponse] = useState<ApiResponse | null>(null);

  const historyQuery = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["api-history", workspaceId],
    queryFn: () => listApiHistory(workspaceId),
  });
  const savedQuery = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["api-saved", workspaceId],
    queryFn: () => listSavedApiRequests(workspaceId),
  });
  const environmentQuery = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["workspace-environment", workspaceId],
    queryFn: () => getWorkspaceEnvironment(workspaceId),
  });

  useEffect(() => {
    if (environmentQuery.data) {
      setEnvVariables(environmentQuery.data.variables);
    }
  }, [environmentQuery.data]);

  const input = useMemo<ApiRequestInput>(
    () => ({
      workspaceId,
      name,
      method,
      url,
      headers,
      query,
      body: method === "GET" || method === "HEAD" ? undefined : body,
      bodyKind: "json",
      timeoutMs: 60_000,
    }),
    [body, headers, method, name, query, url, workspaceId],
  );

  const sendMutation = useMutation({
    mutationFn: sendApiRequest,
    onSuccess: (result) => {
      setResponse(result);
      queryClient.invalidateQueries({ queryKey: ["api-history", workspaceId] });
    },
  });

  const saveMutation = useMutation({
    mutationFn: saveApiRequest,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["api-saved", workspaceId] }),
  });

  const saveEnvironmentMutation = useMutation({
    mutationFn: (variables: KeyValue[]) => updateWorkspaceEnvironment(workspaceId, variables),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["workspace-environment", workspaceId] }),
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    sendMutation.mutate(input);
  }

  function loadSavedRequest(saved: ApiSavedRequest) {
    setName(saved.name);
    setMethod(saved.method);
    setUrl(saved.url);
    setHeaders(parseKeyValues(saved.headersJson));
    setQuery(parseKeyValues(saved.queryJson));
    setBody(saved.body ?? "");
  }

  return (
    <div className="grid h-full min-h-0 grid-cols-[minmax(520px,1fr)_380px] gap-4">
      <form className="flex min-h-0 flex-col rounded-md border border-zinc-200 bg-white" onSubmit={submit}>
        <div className="flex items-center gap-2 border-b border-zinc-200 p-3">
          <select
            className="h-9 rounded-md border border-zinc-200 bg-white px-2 text-sm font-semibold outline-none focus:border-teal-500"
            onChange={(event) => setMethod(event.target.value)}
            value={method}
          >
            {methods.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
          <Input onChange={(event) => setUrl(event.target.value)} value={url} />
          <Button disabled={sendMutation.isPending} type="submit">
            <Send size={15} />
            Send
          </Button>
          <Button
            disabled={saveMutation.isPending}
            onClick={() => saveMutation.mutate(input)}
            type="button"
            variant="outline"
          >
            <Save size={15} />
            Save
          </Button>
        </div>

        <div className="grid min-h-0 flex-1 grid-cols-[320px_minmax(0,1fr)]">
          <div className="space-y-4 overflow-y-auto border-r border-zinc-200 p-3">
            <SavedRequestsList
              items={savedQuery.data ?? []}
              onLoad={loadSavedRequest}
            />
            <FieldGroup title="Request">
              <Input onChange={(event) => setName(event.target.value)} value={name} />
            </FieldGroup>
            <div className="rounded-md border border-zinc-200 p-3">
              <div className="mb-2 flex items-center justify-between">
                <span className="text-xs font-semibold uppercase tracking-wide text-zinc-500">
                  Environment
                </span>
                <Button
                  disabled={saveEnvironmentMutation.isPending}
                  onClick={() => saveEnvironmentMutation.mutate(envVariables)}
                  size="sm"
                  type="button"
                  variant="outline"
                >
                  <Save size={13} />
                  Save
                </Button>
              </div>
              <KeyValueEditor
                items={envVariables}
                onChange={setEnvVariables}
                title="Variables"
              />
            </div>
            <KeyValueEditor items={query} onChange={setQuery} title="Query" />
            <KeyValueEditor items={headers} onChange={setHeaders} title="Headers" />
          </div>

          <div className="flex min-h-0 flex-col">
            <div className="flex h-10 items-center border-b border-zinc-200 px-3 text-sm font-medium">
              Body
            </div>
            <div className="min-h-0 flex-1">
              <Editor
                defaultLanguage="json"
                onChange={(value) => setBody(value ?? "")}
                options={{
                  fontSize: 13,
                  minimap: { enabled: false },
                  scrollBeyondLastLine: false,
                  wordWrap: "on",
                }}
                theme="vs-light"
                value={body}
              />
            </div>
          </div>
        </div>
      </form>

      <div className="flex min-h-0 flex-col gap-4">
        <section className="flex min-h-0 flex-1 flex-col rounded-md border border-zinc-200 bg-white">
          <div className="flex h-10 items-center justify-between border-b border-zinc-200 px-3">
            <div className="flex items-center gap-2 text-sm font-semibold">
              <Braces size={15} />
              Response
            </div>
            {response && (
              <div className="flex items-center gap-2">
                <Badge tone={response.status < 400 ? "green" : "red"}>
                  {response.status}
                </Badge>
                <Badge tone="neutral">{response.durationMs}ms</Badge>
              </div>
            )}
          </div>
          <div className="min-h-0 flex-1">
            <Editor
              defaultLanguage="json"
              options={{
                fontSize: 12,
                minimap: { enabled: false },
                readOnly: true,
                scrollBeyondLastLine: false,
                wordWrap: "on",
              }}
              theme="vs-light"
              value={formatResponseBody(response?.body)}
            />
          </div>
        </section>

        <section className="h-[270px] rounded-md border border-zinc-200 bg-white">
          <div className="flex h-10 items-center justify-between border-b border-zinc-200 px-3">
            <div className="flex items-center gap-2 text-sm font-semibold">
              <History size={15} />
              History
            </div>
            <Badge tone="neutral">{savedQuery.data?.length ?? 0} saved</Badge>
          </div>
          <HistoryTable items={historyQuery.data ?? []} />
        </section>
      </div>
    </div>
  );
}

function SavedRequestsList({
  items,
  onLoad,
}: {
  items: ApiSavedRequest[];
  onLoad: (item: ApiSavedRequest) => void;
}) {
  return (
    <div className="rounded-md border border-zinc-200 p-3">
      <div className="mb-2 flex items-center justify-between">
        <span className="text-xs font-semibold uppercase tracking-wide text-zinc-500">
          Saved
        </span>
        <Badge tone="neutral">{items.length}</Badge>
      </div>
      <div className="max-h-32 space-y-1 overflow-y-auto">
        {items.map((item) => (
          <button
            className="flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs hover:bg-zinc-100"
            key={item.id}
            onClick={() => onLoad(item)}
            type="button"
          >
            <Badge tone="teal">{item.method}</Badge>
            <span className="min-w-0 flex-1 truncate">{item.name}</span>
          </button>
        ))}
        {items.length === 0 && (
          <div className="py-3 text-center text-xs text-zinc-500">No saved requests</div>
        )}
      </div>
    </div>
  );
}

function FieldGroup({ children, title }: { children: React.ReactNode; title: string }) {
  return (
    <label className="block space-y-2">
      <span className="text-xs font-semibold uppercase tracking-wide text-zinc-500">{title}</span>
      {children}
    </label>
  );
}

function KeyValueEditor({
  items,
  onChange,
  title,
}: {
  items: KeyValue[];
  onChange: (items: KeyValue[]) => void;
  title: string;
}) {
  function update(index: number, patch: Partial<KeyValue>) {
    onChange(items.map((item, itemIndex) => (itemIndex === index ? { ...item, ...patch } : item)));
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold uppercase tracking-wide text-zinc-500">{title}</span>
        <Button
          onClick={() => onChange([...items, { key: "", value: "", enabled: true }])}
          size="sm"
          type="button"
          variant="ghost"
        >
          <Plus size={13} />
          Add
        </Button>
      </div>
      <div className="space-y-2">
        {items.map((item, index) => (
          <div className="grid grid-cols-[20px_1fr_1fr] gap-2" key={`${title}-${index}`}>
            <input
              checked={item.enabled}
              className="mt-2 h-4 w-4"
              onChange={(event) => update(index, { enabled: event.target.checked })}
              type="checkbox"
            />
            <Input
              onChange={(event) => update(index, { key: event.target.value })}
              placeholder="Key"
              value={item.key}
            />
            <Input
              onChange={(event) => update(index, { value: event.target.value })}
              placeholder="Value"
              value={item.value}
            />
          </div>
        ))}
      </div>
    </div>
  );
}

function parseKeyValues(value: string): KeyValue[] {
  try {
    const parsed = JSON.parse(value);
    if (Array.isArray(parsed)) {
      return parsed.filter(
        (item): item is KeyValue =>
          typeof item?.key === "string" &&
          typeof item?.value === "string" &&
          typeof item?.enabled === "boolean",
      );
    }
  } catch {
    return [];
  }

  return [];
}

const columnHelper = createColumnHelper<ApiHistoryItem>();

function HistoryTable({ items }: { items: ApiHistoryItem[] }) {
  const columns = useMemo(
    () => [
      columnHelper.accessor("method", {
        cell: (info) => <Badge tone="teal">{info.getValue()}</Badge>,
        header: "Method",
      }),
      columnHelper.accessor("status", {
        cell: (info) => {
          const status = info.getValue();
          return status ? <Badge tone={status < 400 ? "green" : "red"}>{status}</Badge> : "-";
        },
        header: "Status",
      }),
      columnHelper.accessor("url", {
        cell: (info) => <span className="block max-w-[190px] truncate">{info.getValue()}</span>,
        header: "URL",
      }),
      columnHelper.accessor("durationMs", {
        cell: (info) => {
          const value = info.getValue();
          return value ? `${value}ms` : "-";
        },
        header: "Time",
      }),
    ],
    [],
  );
  const table = useReactTable({
    columns,
    data: items,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <div className="h-[228px] overflow-auto">
      <table className="w-full text-left text-xs">
        <thead className="sticky top-0 bg-zinc-50 text-zinc-500">
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <th className="border-b border-zinc-200 px-3 py-2 font-medium" key={header.id}>
                  {flexRender(header.column.columnDef.header, header.getContext())}
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row) => (
            <tr className="border-b border-zinc-100" key={row.id}>
              {row.getVisibleCells().map((cell) => (
                <td className="px-3 py-2" key={cell.id}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {items.length === 0 && (
        <div className="flex h-24 items-center justify-center text-sm text-zinc-500">
          No requests yet
        </div>
      )}
    </div>
  );
}

function SshPanel() {
  return (
    <div className="grid h-full min-h-0 grid-cols-[360px_minmax(0,1fr)] gap-4">
      <section className="rounded-md border border-zinc-200 bg-white p-4">
        <div className="mb-4 flex items-center gap-2 text-sm font-semibold">
          <TerminalSquare size={16} />
          SSH Sessions
        </div>
        <div className="space-y-3">
          <Input placeholder="Host" value="example.internal" readOnly />
          <Input placeholder="User" value="deploy" readOnly />
          <div className="grid grid-cols-2 gap-2">
            <Button type="button" variant="outline">
              <KeyRound size={15} />
              Password
            </Button>
            <Button type="button" variant="outline">
              <KeyRound size={15} />
              Private key
            </Button>
          </div>
          <Badge tone="amber">reserved backend</Badge>
        </div>
      </section>
      <TerminalPreview />
    </div>
  );
}

function DatabasePanel() {
  return (
    <div className="grid h-full min-h-0 grid-cols-[320px_minmax(0,1fr)] gap-4">
      <section className="rounded-md border border-zinc-200 bg-white p-4">
        <div className="mb-4 flex items-center gap-2 text-sm font-semibold">
          <Database size={16} />
          Connections
        </div>
        <div className="space-y-2">
          {["PostgreSQL", "MySQL / MariaDB", "SQLite"].map((driver) => (
            <button
              className="flex h-9 w-full items-center justify-between rounded-md border border-zinc-200 px-3 text-sm hover:bg-zinc-50"
              key={driver}
              type="button"
            >
              {driver}
              <Badge tone="neutral">MVP</Badge>
            </button>
          ))}
        </div>
      </section>
      <section className="flex min-h-0 flex-col rounded-md border border-zinc-200 bg-white">
        <div className="flex h-10 items-center gap-2 border-b border-zinc-200 px-3 text-sm font-semibold">
          <Clock size={15} />
          SQL Editor
        </div>
        <div className="min-h-0 flex-1">
          <Editor
            defaultLanguage="sql"
            options={{
              fontSize: 13,
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
            }}
            value={"select *\nfrom workspace_connections\nlimit 100;"}
          />
        </div>
      </section>
    </div>
  );
}

function formatResponseBody(body?: string) {
  if (!body) {
    return "";
  }

  try {
    return JSON.stringify(JSON.parse(body), null, 2);
  } catch {
    return body;
  }
}

export default App;
