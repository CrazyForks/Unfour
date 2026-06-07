import { Activity, Minus } from "lucide-react";
import { BottomPanel, IconButton } from "@unfour/ui";

export function BottomPanelPlaceholder({
  collapsed,
  height,
  onCollapse,
  onHeightChange,
}: {
  collapsed: boolean;
  height: number;
  onCollapse: () => void;
  onHeightChange: (height: number) => void;
}) {
  return (
    <BottomPanel
      collapsed={collapsed}
      height={height}
      onHeightChange={onHeightChange}
      resizable
    >
      <div className="flex h-[var(--u-size-section-toolbar)] items-center justify-between border-b border-[var(--u-color-border)] px-2">
        <div className="flex items-center gap-2 text-[12px] font-semibold text-[var(--u-color-text)]">
          <Activity size={14} />
          Diagnostics
        </div>
        <IconButton label="Collapse bottom panel" onClick={onCollapse}>
          <Minus size={14} />
        </IconButton>
      </div>
      <div className="p-2 text-[12px] text-[var(--u-color-text-muted)]">
        Local activity and module diagnostics will appear here.
      </div>
    </BottomPanel>
  );
}
