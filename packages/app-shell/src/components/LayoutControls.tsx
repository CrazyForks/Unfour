import { PanelBottom, PanelLeft, PanelRight } from "lucide-react";
import { IconButton, cn, useI18n } from "@unfour/ui";

const activeClass = "bg-[var(--u-color-primary-soft)] text-[var(--u-color-primary)]";
const statusBarButtonClass =
  "h-5 w-5 [&>span]:bottom-full [&>span]:top-auto [&>span]:mb-1 [&>span]:mt-0";

export function LayoutControls({
  bottomPanelCollapsed,
  onToggleBottomPanel,
  onToggleInspector,
  onToggleSidebar,
  rightInspectorCollapsed,
  sidebarCollapsed,
}: {
  bottomPanelCollapsed: boolean;
  onToggleBottomPanel: () => void;
  onToggleInspector: () => void;
  onToggleSidebar: () => void;
  rightInspectorCollapsed: boolean;
  sidebarCollapsed: boolean;
}) {
  const { t } = useI18n();

  return (
    <div className="flex items-center gap-px border-l border-[var(--u-color-border)] pl-2">
      <IconButton
        aria-pressed={!sidebarCollapsed}
        className={cn(statusBarButtonClass, !sidebarCollapsed && activeClass)}
        label={sidebarCollapsed ? t("app.sidebar.expand") : t("app.sidebar.collapse")}
        onClick={onToggleSidebar}
        size="compact"
      >
        <PanelLeft size={13} />
      </IconButton>
      <IconButton
        aria-pressed={!bottomPanelCollapsed}
        className={cn(statusBarButtonClass, !bottomPanelCollapsed && activeClass)}
        label={t("app.titlebar.toggleBottomPanel")}
        onClick={onToggleBottomPanel}
        size="compact"
      >
        <PanelBottom size={13} />
      </IconButton>
      <IconButton
        aria-pressed={!rightInspectorCollapsed}
        className={cn(statusBarButtonClass, !rightInspectorCollapsed && activeClass)}
        label={t("app.titlebar.toggleInspector")}
        onClick={onToggleInspector}
        size="compact"
      >
        <PanelRight size={13} />
      </IconButton>
    </div>
  );
}
