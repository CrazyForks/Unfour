import { Save, Send } from "lucide-react";
import { Badge, Button, Input, cn } from "@unfour/ui";
import {
  getTabSaveState,
  methodToneClass,
  requestTabVisualState,
  type ApiRequestTab,
} from "../model/request-tabs";
import { methods } from "../hooks/useApiRequest";
import { RequestActionsMenu } from "./RequestActionsMenu";

export function ApiRequestBar({
  onDelete,
  onDuplicate,
  onExport,
  onImport,
  onSave,
  onSend,
  onUpdate,
  tab,
}: {
  onDelete: () => void;
  onDuplicate: () => void;
  onExport: () => void;
  onImport: () => void;
  onSave: () => void;
  onSend: () => void;
  onUpdate: (patch: Partial<ApiRequestTab["draft"]>) => void;
  tab: ApiRequestTab;
}) {
  const saveState = getTabSaveState(tab);
  const visualState = requestTabVisualState(tab);
  return (
    <div className="flex min-h-[48px] shrink-0 items-center gap-2 border-b border-[var(--u-color-border)] bg-[var(--u-color-surface)] px-3 py-2">
      <div className="flex min-w-0 flex-1 items-stretch overflow-hidden rounded-[var(--u-radius-md)] border border-[var(--u-color-input)] bg-[var(--u-color-bg)] focus-within:border-[var(--u-color-focus)]">
        <select
          aria-label="HTTP method"
          className={cn(
            "h-[var(--u-size-input)] w-[98px] border-0 border-r border-[var(--u-color-border)] bg-transparent px-2 text-[12px] font-bold uppercase outline-none",
            methodToneClass(tab.draft.method),
          )}
          onChange={(event) => onUpdate({ method: event.target.value })}
          value={tab.draft.method}
        >
          {methods.map((method) => (
            <option key={method}>{method}</option>
          ))}
        </select>
        <Input
          aria-label="Request URL"
          className="min-w-0 flex-1 border-0 bg-transparent font-mono focus:border-0"
          onChange={(event) => onUpdate({ url: event.target.value })}
          placeholder="https://api.example.com/resource"
          value={tab.draft.url}
        />
      </div>
      <Button
        disabled={tab.sending || !tab.draft.url.trim()}
        size="sm"
        onClick={onSend}
        type="button"
      >
        <Send size={14} />
        {tab.sending ? "Sending" : "Send"}
      </Button>
      <Button
        disabled={tab.saving}
        size="sm"
        onClick={onSave}
        type="button"
        variant="outline"
      >
        <Save size={14} />
        {tab.saving ? "Saving" : "Save"}
      </Button>
      <Badge
        tone={
          visualState === "failed"
            ? "red"
            : visualState === "success" || saveState === "saved"
              ? "green"
              : "amber"
        }
      >
        {tab.saving ? "saving" : visualState}
      </Badge>
      <RequestActionsMenu
        canDelete={Boolean(tab.savedRequestId)}
        canDuplicate={Boolean(tab.savedRequestId)}
        onDelete={onDelete}
        onDuplicate={onDuplicate}
        onExport={onExport}
        onImport={onImport}
      />
      {tab.saveError && (
        <span
          className="max-w-[180px] truncate text-[12px] text-[var(--u-color-danger)]"
          title={tab.saveError}
        >
          {tab.saveError}
        </span>
      )}
    </div>
  );
}
