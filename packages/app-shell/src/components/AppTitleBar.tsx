import {
  Languages,
  Moon,
  PanelBottom,
  PanelLeft,
  PanelRight,
  Search,
  Sun,
} from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Workspace } from "@unfour/command-client";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  GlobalToolbar,
  IconButton,
  cn,
  getLocaleLabel,
  useI18n,
  useTheme,
  type Locale,
} from "@unfour/ui";
import { isTauriRuntime } from "./module-helpers";
import { WindowControls } from "./WindowControls";
import { WorkspaceMenu } from "./WorkspaceMenu";

export function AppTitleBar({
  activeWorkspace,
  bottomPanelCollapsed,
  healthReady,
  onActivateWorkspace,
  onOpenCommandPalette,
  onToggleBottomPanel,
  onToggleInspector,
  onToggleSidebar,
  rightInspectorCollapsed,
  sidebarCollapsed,
  syncStrategy,
  workspaces,
}: {
  activeWorkspace?: Workspace;
  bottomPanelCollapsed: boolean;
  healthReady: boolean;
  onActivateWorkspace: (workspaceId: string) => void;
  onOpenCommandPalette: () => void;
  onToggleBottomPanel: () => void;
  onToggleInspector: () => void;
  onToggleSidebar: () => void;
  rightInspectorCollapsed: boolean;
  sidebarCollapsed: boolean;
  syncStrategy: string;
  workspaces: Workspace[];
}) {
  const { t } = useI18n();

  async function dragWindow(event: React.MouseEvent<HTMLDivElement>) {
    if ((event.target as HTMLElement).closest("button,input,select,a")) {
      return;
    }
    if (event.button !== 0 || !isTauriRuntime()) {
      return;
    }

    await getCurrentWindow().startDragging();
  }

  return (
    <GlobalToolbar
      center={
        <button
          className="flex h-7 w-[min(440px,100%)] min-w-[160px] items-center gap-2 rounded-[var(--u-radius-md)] border border-[var(--u-color-border)] bg-[var(--u-color-surface)] px-2.5 text-left text-[12px] text-[var(--u-color-text-soft)] transition-colors duration-150 hover:border-[var(--u-color-border-strong)] hover:bg-[var(--u-color-surface-hover)] hover:text-[var(--u-color-text)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:color-mix(in_srgb,var(--u-color-focus)_32%,transparent)]"
          onClick={onOpenCommandPalette}
          type="button"
        >
          <Search size={14} />
          <span className="min-w-0 flex-1 truncate">
            {t("app.commandPalette.placeholder")}
          </span>
          <span className="flex shrink-0 items-center gap-1">
            <kbd className="rounded border border-[var(--u-color-border)] bg-[var(--u-color-surface-muted)] px-1.5 py-0.5 text-[10px] font-semibold text-[var(--u-color-text-muted)]">
              Ctrl
            </kbd>
            <kbd className="rounded border border-[var(--u-color-border)] bg-[var(--u-color-surface-muted)] px-1.5 py-0.5 text-[10px] font-semibold text-[var(--u-color-text-muted)]">
              K
            </kbd>
          </span>
        </button>
      }
      left={
        <WorkspaceMenu
          activeWorkspace={activeWorkspace}
          onActivateWorkspace={onActivateWorkspace}
          workspaces={workspaces}
        />
      }
      onDragRegionMouseDown={dragWindow}
      right={
        <>
          <ThemeToggle />
          <div className="mx-1 h-5 w-px bg-[var(--u-color-border)]" />
          <IconButton
            aria-pressed={!sidebarCollapsed}
            className={
              !sidebarCollapsed
                ? "bg-[var(--u-color-primary-soft)] text-[var(--u-color-primary)]"
                : undefined
            }
            label={
              sidebarCollapsed
                ? t("app.sidebar.expand")
                : t("app.sidebar.collapse")
            }
            onClick={onToggleSidebar}
          >
            <PanelLeft size={15} />
          </IconButton>
          <IconButton
            aria-pressed={!bottomPanelCollapsed}
            className={
              !bottomPanelCollapsed
                ? "bg-[var(--u-color-primary-soft)] text-[var(--u-color-primary)]"
                : undefined
            }
            label={t("app.titlebar.toggleBottomPanel")}
            onClick={onToggleBottomPanel}
          >
            <PanelBottom size={15} />
          </IconButton>
          <IconButton
            aria-pressed={!rightInspectorCollapsed}
            className={
              !rightInspectorCollapsed
                ? "bg-[var(--u-color-primary-soft)] text-[var(--u-color-primary)]"
                : undefined
            }
            label={t("app.titlebar.toggleInspector")}
            onClick={onToggleInspector}
          >
            <PanelRight size={15} />
          </IconButton>
          <div className="mx-1 h-5 w-px bg-[var(--u-color-border)]" />
          <span
            className="flex h-7 items-center gap-1.5 px-1 text-[12px] text-[var(--u-color-text-muted)]"
            title={`${healthReady ? t("app.status.storageReady") : t("app.status.checkingStorage")} · ${syncStrategy}`}
          >
            <span className="h-1.5 w-1.5 rounded-full bg-[var(--u-color-success)] shadow-[0_0_0_3px_var(--u-color-success-soft)]" />
            {healthReady ? t("app.status.ready") : t("app.status.checkingStorage")}
          </span>
          <LanguageMenu />
          <WindowControls />
        </>
      }
    />
  );
}

function ThemeToggle() {
  const { t } = useI18n();
  const { setTheme, theme } = useTheme();
  const options: { icon: typeof Sun; label: string; value: "light" | "dark" }[] = [
    { icon: Sun, label: t("app.theme.light"), value: "light" },
    { icon: Moon, label: t("app.theme.dark"), value: "dark" },
  ];

  return (
    <div
      aria-label={t("app.theme.label")}
      className="inline-flex items-center gap-px rounded-full border border-[var(--u-color-border)] bg-[var(--u-color-surface)] p-0.5"
      role="group"
    >
      {options.map((option) => {
        const Icon = option.icon;
        const active = theme === option.value;
        return (
          <button
            aria-label={option.label}
            aria-pressed={active}
            className={cn(
              "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-[11px] font-semibold transition-colors duration-150",
              active
                ? "bg-[var(--u-color-primary)] text-[var(--u-color-primary-foreground)]"
                : "text-[var(--u-color-text-soft)] hover:text-[var(--u-color-text)]",
            )}
            key={option.value}
            onClick={() => setTheme(option.value)}
            title={option.label}
            type="button"
          >
            <Icon size={13} />
            {option.label}
          </button>
        );
      })}
    </div>
  );
}

function LanguageMenu() {
  const { locale, locales, setLocale, t } = useI18n();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <IconButton label={t("app.language.label")}>
          <Languages size={15} />
        </IconButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {locales.map((item) => (
          <DropdownMenuItem
            className={cn(
              "cursor-pointer",
              item === locale &&
                "bg-[var(--u-color-primary-soft)] text-[var(--u-color-primary)]",
            )}
            key={item}
            onSelect={() => setLocale(item as Locale)}
          >
            {getLocaleLabel(item)}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
