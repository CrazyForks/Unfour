import { Braces, Folder, Send } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  ContextMenuItem,
  SidebarRow,
  SidebarSection,
  TreeView,
  type TreeViewItem,
} from "@unfour/ui";
import {
  deleteApiRequest,
  duplicateApiRequest,
  listApiHistory,
  listSavedApiRequests,
  type ApiSavedRequest,
} from "@unfour/command-client";
import { groupSavedRequests, parseKeyValues } from "../request-utils";
import type { ApiOpenIntent } from "../model/types";
import { ApiHistoryTree } from "./ApiHistoryTree";

export function ApiCollectionTree({
  active,
  collapsed,
  onOpenClient,
  onOpenIntent,
  selectedId,
  workspaceId,
}: {
  active: boolean;
  collapsed: boolean;
  onOpenClient: () => void;
  onOpenIntent: (intent: ApiOpenIntent) => void;
  selectedId: string | null;
  workspaceId: string;
}) {
  const queryClient = useQueryClient();
  const savedQuery = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["api-saved", workspaceId],
    queryFn: () => listSavedApiRequests(workspaceId),
  });
  const historyQuery = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["api-history", workspaceId],
    queryFn: () => listApiHistory(workspaceId),
  });
  const duplicateMutation = useMutation({
    mutationFn: (requestId: string) => duplicateApiRequest(workspaceId, requestId),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["api-saved", workspaceId] }),
  });
  const deleteMutation = useMutation({
    mutationFn: (requestId: string) => deleteApiRequest(workspaceId, requestId),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["api-saved", workspaceId] }),
  });
  const collectionItems: TreeViewItem[] = groupSavedRequests(
    savedQuery.data ?? [],
  ).map((group) => ({
    id: `folder:${group.folder}`,
    icon: <Folder size={13} />,
    label: group.folder,
    children: group.items.map((request) =>
      requestTreeItem(
        request,
        onOpenIntent,
        duplicateMutation.mutate,
        deleteMutation.mutate,
      ),
    ),
  }));

  if (collapsed) {
    return (
      <SidebarRow active={active} onClick={onOpenClient} title="REST Client">
        <Send size={14} />
      </SidebarRow>
    );
  }

  return (
    <div className="space-y-3">
      <SidebarSection title="Collections">
        <SidebarRow active={active && !selectedId} onClick={onOpenClient}>
          <Send size={14} />
          <span>New Request</span>
        </SidebarRow>
        {collectionItems.length ? (
          <TreeView
            defaultExpandedIds={collectionItems.map((item) => item.id)}
            items={collectionItems}
            onSelect={(item) => {
              if (item.id.startsWith("request:")) {
                onOpenIntent({
                  kind: "saved",
                  nonce: Date.now(),
                  requestId: item.id.slice("request:".length),
                });
              }
            }}
            selectedId={selectedId ? `request:${selectedId}` : null}
          />
        ) : (
          <SidebarEmpty>No saved requests</SidebarEmpty>
        )}
      </SidebarSection>
      <SidebarSection title="Environments">
        <SidebarEmpty>Workspace environment</SidebarEmpty>
      </SidebarSection>
      <SidebarSection title="History">
        {(historyQuery.data?.length ?? 0) > 0 ? (
          <ApiHistoryTree
            items={historyQuery.data ?? []}
            onOpenIntent={onOpenIntent}
          />
        ) : (
          <SidebarEmpty>Send a request to build history</SidebarEmpty>
        )}
      </SidebarSection>
    </div>
  );
}

function requestTreeItem(
  request: ApiSavedRequest,
  onOpenIntent: (intent: ApiOpenIntent) => void,
  duplicate: (requestId: string) => void,
  remove: (requestId: string) => void,
): TreeViewItem {
  const open = (action: "open" | "send" = "open") =>
    onOpenIntent({
      action,
      kind: "saved",
      nonce: Date.now(),
      requestId: request.id,
    });
  return {
    id: `request:${request.id}`,
    icon: <Braces size={13} />,
    label: request.name,
    title: request.url,
    meta: <MethodMeta method={request.method} />,
    contextMenu: (
      <>
        <ContextMenuItem onSelect={() => open()}>Open</ContextMenuItem>
        <ContextMenuItem disabled>Open in New Tab (unique tab)</ContextMenuItem>
        <ContextMenuItem onSelect={() => open("send")}>Send</ContextMenuItem>
        <ContextMenuItem disabled>Rename (not available in this phase)</ContextMenuItem>
        <ContextMenuItem onSelect={() => duplicate(request.id)}>
          Duplicate
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={() => void navigator.clipboard?.writeText(request.url)}
        >
          Copy URL
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => exportRequest(request)}>Export</ContextMenuItem>
        <ContextMenuItem
          className="text-[var(--u-color-danger)]"
          onSelect={() => remove(request.id)}
        >
          Delete
        </ContextMenuItem>
      </>
    ),
  };
}

function MethodMeta({ method }: { method: string }) {
  return (
    <span className="rounded-[var(--u-radius-sm)] bg-[var(--u-color-surface-muted)] px-1 text-[10px] font-semibold uppercase">
      {method}
    </span>
  );
}

function SidebarEmpty({ children }: { children: React.ReactNode }) {
  return (
    <div className="rounded-[var(--u-radius-sm)] border border-dashed border-[var(--u-color-border)] px-2 py-2 text-[12px] text-[var(--u-color-text-muted)]">
      {children}
    </div>
  );
}

function exportRequest(request: ApiSavedRequest) {
  const value = {
    name: request.name,
    folderPath: request.folderPath,
    method: request.method,
    url: request.url,
    headers: parseKeyValues(request.headersJson),
    query: parseKeyValues(request.queryJson),
    body: request.body,
    bodyKind: request.bodyKind,
  };
  const href = URL.createObjectURL(
    new Blob([JSON.stringify(value, null, 2)], {
      type: "application/json;charset=utf-8",
    }),
  );
  const link = document.createElement("a");
  link.href = href;
  link.download = `${request.name.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}.json`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(href);
}
