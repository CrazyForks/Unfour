import { Loader2, X } from "lucide-react";
import { cn } from "@unfour/ui";
import {
  getTabSaveState,
  requestTabTitle,
  requestTabVisualState,
  type ApiRequestTab,
} from "../model/request-tabs";

export function ApiRequestTabs({
  activeId,
  onClose,
  onNew,
  onSelect,
  tabs,
}: {
  activeId: string | null;
  onClose: (tab: ApiRequestTab) => void;
  onNew: () => void;
  onSelect: (tabId: string) => void;
  tabs: ApiRequestTab[];
}) {
  return (
    <div
      className="flex h-[var(--u-size-tabbar)] shrink-0 items-end overflow-x-auto border-b border-[var(--u-color-border)] bg-[var(--u-color-surface-subtle)] px-2"
      role="tablist"
    >
      {tabs.map((tab) => {
        const active = tab.id === activeId;
        const saveState = getTabSaveState(tab);
        const visualState = requestTabVisualState(tab);
        return (
          <div
            className={cn(
              "group flex h-[30px] min-w-[132px] max-w-[220px] items-center gap-1 rounded-t-[var(--u-radius-sm)] border border-transparent px-2 text-[12px]",
              active
                ? "border-[var(--u-color-border)] border-b-[var(--u-color-surface)] bg-[var(--u-color-surface)] text-[var(--u-color-text)]"
                : "text-[var(--u-color-text-muted)] hover:bg-[var(--u-color-surface-hover)]",
            )}
            key={tab.id}
          >
            <button
              aria-selected={active}
              className="flex min-w-0 flex-1 items-center gap-1.5"
              onClick={() => onSelect(tab.id)}
              role="tab"
              title={`${requestTabTitle(tab)} · ${visualState}`}
              type="button"
            >
              {(saveState === "dirty" || saveState === "unsaved") && (
                <span
                  aria-label={saveState}
                  className="h-1.5 w-1.5 shrink-0 rounded-full bg-[var(--u-color-warning)]"
                />
              )}
              {tab.sending && (
                <Loader2
                  aria-label="sending"
                  className="shrink-0 animate-spin text-[var(--u-color-primary)]"
                  size={12}
                />
              )}
              <span className="truncate">{requestTabTitle(tab)}</span>
            </button>
            <button
              aria-label={`Close ${requestTabTitle(tab)}`}
              className="grid h-5 w-5 shrink-0 place-items-center rounded-[var(--u-radius-sm)] text-[var(--u-color-text-soft)] hover:bg-[var(--u-color-surface-hover)]"
              onClick={() => onClose(tab)}
              type="button"
            >
              <X size={12} />
            </button>
          </div>
        );
      })}
      <button
        aria-label="New request"
        className="mb-1 ml-1 grid h-6 w-6 shrink-0 place-items-center rounded-[var(--u-radius-sm)] text-[var(--u-color-text-muted)] hover:bg-[var(--u-color-surface-hover)]"
        onClick={onNew}
        title="New request"
        type="button"
      >
        +
      </button>
    </div>
  );
}
