import { useState } from "react";
import { Check, ChevronDown, Layers, Settings2 } from "lucide-react";
import {
  Button,
  Popover,
  PopoverContent,
  PopoverTrigger,
  cn,
  useI18n,
} from "@unfour/ui";
import { useApiEnvironments } from "../hooks/useApiEnvironments";

/**
 * Request-bar environment control: a compact switcher for send resolution.
 * Environment creation/editing lives in the dedicated API Environments tab.
 */
export function EnvironmentControl({
  activeEnvironmentId,
  onManageEnvironments,
  onSelectEnvironment,
  workspaceId,
}: {
  activeEnvironmentId: string | null;
  onManageEnvironments: () => void;
  onSelectEnvironment: (environmentId: string | null) => void;
  workspaceId: string;
}) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const { activeEnvironment, environments } = useApiEnvironments(workspaceId);

  function close() {
    setOpen(false);
  }

  return (
    <Popover
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) {
          close();
        }
      }}
      open={open}
    >
      <PopoverTrigger asChild>
        <button
          aria-label={t("api.environment.active")}
          className="flex h-[var(--u-size-input)] max-w-[170px] shrink-0 items-center gap-1.5 rounded-[var(--u-radius-md)] border border-[var(--u-color-border)] bg-[var(--u-color-surface)] px-2 text-[12px] text-[var(--u-color-text)] outline-none transition-colors hover:border-[var(--u-color-border-strong)] focus:border-[var(--u-color-focus)]"
          title={t("api.environment.active")}
          type="button"
        >
          <Layers className="shrink-0 text-[var(--u-color-text-muted)]" size={13} />
          <span className="min-w-0 flex-1 truncate text-left">
            {activeEnvironment?.name ?? t("api.environment.none")}
          </span>
          <ChevronDown className="shrink-0 text-[var(--u-color-text-soft)]" size={13} />
        </button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-[280px]">
        <div className="flex flex-col py-1">
          <EnvironmentRow
            active={activeEnvironmentId === null}
            label={t("api.environment.none")}
            muted
            onActivate={() => {
              onSelectEnvironment(null);
              close();
            }}
          />
          {environments.length > 0 && (
            <div className="my-1 border-t border-[var(--u-color-border)]" />
          )}
          {environments.map((environment) => (
            <EnvironmentRow
              active={activeEnvironmentId === environment.id}
              key={environment.id}
              label={environment.name}
              onActivate={() => {
                onSelectEnvironment(environment.id);
                close();
              }}
            />
          ))}
          <div className="mt-1 space-y-1 border-t border-[var(--u-color-border)] px-2 pt-1.5">
            <Button
              className="w-full justify-start"
              onClick={() => {
                onManageEnvironments();
                close();
              }}
              size="sm"
              type="button"
              variant="ghost"
            >
              <Settings2 size={13} />
              {t("api.environment.manageEnvironments")}
            </Button>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}

function EnvironmentRow({
  active,
  label,
  muted,
  onActivate,
}: {
  active: boolean;
  label: string;
  muted?: boolean;
  onActivate: () => void;
}) {
  return (
    <div
      className={cn(
        "group flex items-center gap-1 px-1.5",
        active && "bg-[var(--u-color-primary-soft)]",
      )}
    >
      <button
        className="flex min-w-0 flex-1 items-center gap-2 rounded-[var(--u-radius-sm)] px-1.5 py-1.5 text-left hover:bg-[var(--u-color-surface-hover)]"
        onClick={onActivate}
        type="button"
      >
        <Check
          className={cn(
            "shrink-0",
            active ? "text-[var(--u-color-primary)]" : "text-transparent",
          )}
          size={14}
        />
        <span
          className={cn(
            "min-w-0 flex-1 truncate text-[12px]",
            muted
              ? "text-[var(--u-color-text-muted)]"
              : "font-medium text-[var(--u-color-text)]",
            active && "text-[var(--u-color-primary)]",
          )}
        >
          {label}
        </span>
      </button>
    </div>
  );
}
