import {
  Activity,
  ChevronLeft,
  ChevronRight,
  Home,
  Maximize2,
  MoreHorizontal,
  PanelLeftClose,
  Search,
} from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Workspace } from "@unfour/command-client";
import { Badge, GlobalToolbar, IconButton } from "@unfour/ui";
import { isTauriRuntime } from "./utils";
import { WindowControls } from "./WindowControls";
import { WorkspaceMenu } from "./WorkspaceMenu";

export function AppTitleBar({
  activeWorkspace,
  healthReady,
  onActivateWorkspace,
  onOpenCommandPalette,
  onToggleBottomPanel,
  onToggleInspector,
  syncStrategy,
  workspaces,
}: {
  activeWorkspace?: Workspace;
  healthReady: boolean;
  onActivateWorkspace: (workspaceId: string) => void;
  onOpenCommandPalette: () => void;
  onToggleBottomPanel: () => void;
  onToggleInspector: () => void;
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
            onActivateWorkspace={onActivateWorkspace}
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
