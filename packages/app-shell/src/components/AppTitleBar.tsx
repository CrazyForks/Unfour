import { Languages, Moon, Sun } from "lucide-react";
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
  healthReady,
  onActivateWorkspace,
  syncStrategy,
  workspaces,
}: {
  activeWorkspace?: Workspace;
  healthReady: boolean;
  onActivateWorkspace: (workspaceId: string) => void;
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
