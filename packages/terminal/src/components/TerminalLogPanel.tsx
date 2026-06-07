import { Activity, Copy, Download, Minus } from "lucide-react";
import { useMutation } from "@tanstack/react-query";
import { exportSshLog } from "@unfour/command-client";
import {
  BottomPanel,
  EmptyState,
  IconButton,
  StatusBadge,
  Toolbar,
  ToolbarGroup,
} from "@unfour/ui";
import { useTerminalStore, redactTerminalLog } from "../model/terminal-state";

export function TerminalLogPanel({
  collapsed,
  height,
  onCollapse,
  onHeightChange,
  workspaceId,
}: {
  collapsed?: boolean;
  height?: number;
  onCollapse: () => void;
  onHeightChange: (height: number) => void;
  workspaceId: string;
}) {
  const activeSessionId = useTerminalStore((state) => state.activeSessionId);
  const events = useTerminalStore((state) => state.terminalEvents);
  const exportedLog = useTerminalStore((state) => state.exportedLog);
  const setExportedLog = useTerminalStore((state) => state.setExportedLog);
  const logLines = events.map(
    (event) => `[${event.createdAt}] ${event.kind} ${redactTerminalLog(event.data).trim()}`,
  );
  const visibleLog = redactTerminalLog(exportedLog ?? logLines.join("\n"));

  const exportMutation = useMutation({
    mutationFn: () => exportSshLog({ workspaceId, sessionId: activeSessionId ?? "" }),
    onSuccess: (log) => setExportedLog(log.content),
  });

  return (
    <BottomPanel
      className="flex flex-col"
      collapsed={collapsed}
      height={height}
      onHeightChange={onHeightChange}
      resizable
    >
      <Toolbar>
        <ToolbarGroup>
          <Activity size={14} />
          <span className="text-[12px] font-semibold text-[var(--u-color-text)]">
            Connection Events
          </span>
          <StatusBadge>{events.length}</StatusBadge>
          {exportMutation.error && <StatusBadge tone="danger">Export failed</StatusBadge>}
        </ToolbarGroup>
        <ToolbarGroup>
          <IconButton
            disabled={!visibleLog}
            label="Copy terminal logs"
            onClick={() => void navigator.clipboard?.writeText(visibleLog)}
          >
            <Copy size={14} />
          </IconButton>
          <IconButton
            disabled={!activeSessionId || exportMutation.isPending}
            label="Export terminal logs"
            onClick={() => exportMutation.mutate()}
          >
            <Download size={14} />
          </IconButton>
          <IconButton label="Collapse bottom panel" onClick={onCollapse}>
            <Minus size={14} />
          </IconButton>
        </ToolbarGroup>
      </Toolbar>
      <div className="min-h-0 flex-1 overflow-auto p-2">
        {visibleLog ? (
          <pre className="min-h-full whitespace-pre-wrap break-words font-mono text-[12px] leading-5 text-[var(--u-color-text-muted)]">
            {visibleLog}
          </pre>
        ) : (
          <EmptyState className="h-full min-h-0">No connection events yet.</EmptyState>
        )}
      </div>
    </BottomPanel>
  );
}
