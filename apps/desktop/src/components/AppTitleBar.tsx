import {
  Activity,
  MoreHorizontal,
  Settings,
} from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Workspace } from "@unfour/command-client";
import {
  GlobalToolbar,
  IconButton,
  Select,
  getLocaleLabel,
  useI18n,
  type Locale,
} from "@unfour/ui";
import { isTauriRuntime } from "./module-helpers";
import { WindowControls } from "./WindowControls";
import { WorkspaceMenu } from "./WorkspaceMenu";

export function AppTitleBar({
  activeWorkspace,
  healthReady,
  onActivateWorkspace,
  onOpenCommandPalette,
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
  const { locale, locales, setLocale, t } = useI18n();

  async function dragWindow(event: React.MouseEvent<HTMLDivElement>) {
    if (event.button !== 0 || !isTauriRuntime()) {
      return;
    }

    await getCurrentWindow().startDragging();
  }

  return (
    <GlobalToolbar
      center={<div className="h-full min-w-0 flex-1" />}
      className="bg-[var(--u-color-surface-subtle)]"
      left={
        <>
          <div className="flex h-[26px] w-[26px] items-center justify-center rounded-[var(--u-radius-sm)] bg-[var(--u-color-primary)] text-[var(--u-color-primary-foreground)]">
            <Activity size={15} />
          </div>
          <span className="mr-3 text-[13px] font-semibold text-[var(--u-color-text)]">
            Unfour
          </span>
          <div className="mx-1 h-5 w-px bg-[var(--u-color-border)]" />
          <WorkspaceMenu
            activeWorkspace={activeWorkspace}
            className="ml-1"
            onActivateWorkspace={onActivateWorkspace}
            workspaces={workspaces}
          />
        </>
      }
      onDragRegionMouseDown={dragWindow}
      right={
        <>
          <span
            className="rounded-[var(--u-radius-sm)] border border-[var(--u-color-border)] px-2 py-0.5 font-mono text-[11px] text-[var(--u-color-text-muted)]"
            title={`${healthReady ? t("app.status.storageReady") : t("app.status.checkingStorage")} · ${syncStrategy}`}
          >
            v0.1.0
          </span>
          <Select
            aria-label={t("app.language.label")}
            className="h-7 w-[116px] px-1 text-[11px]"
            onChange={(event) => setLocale(event.target.value as Locale)}
            options={locales.map((item) => ({
              label: getLocaleLabel(item),
              value: item,
            }))}
            value={locale}
          />
          <button
            className="flex h-7 w-7 items-center justify-center rounded-full bg-[var(--u-color-surface-muted)] text-[11px] font-semibold text-[var(--u-color-text)]"
            type="button"
          >
            UF
          </button>
          <IconButton label={t("app.titlebar.settings")} onClick={onOpenCommandPalette}>
            <Settings size={15} />
          </IconButton>
          <IconButton label={t("app.titlebar.moreActions")} onClick={onOpenCommandPalette}>
            <MoreHorizontal size={16} />
          </IconButton>
          <WindowControls />
        </>
      }
    />
  );
}
