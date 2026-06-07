import * as React from "react";
import { cn } from "./utils";

export type StatusTone = "neutral" | "success" | "warning" | "danger";

const toneClass: Record<StatusTone, string> = {
  danger: "bg-[var(--u-color-danger-soft)] text-[var(--u-color-danger)]",
  neutral: "bg-[var(--u-color-surface-muted)] text-[var(--u-color-text-muted)]",
  success: "bg-[var(--u-color-success-soft)] text-[var(--u-color-success)]",
  warning: "bg-[var(--u-color-warning-soft)] text-[var(--u-color-warning)]",
};

export function StatusBadge({
  children,
  className,
  tone = "neutral",
}: {
  children: React.ReactNode;
  className?: string;
  tone?: StatusTone;
}) {
  return (
    <span
      className={cn(
        "inline-flex h-5 max-w-full items-center rounded-[var(--u-radius-sm)] px-1.5 text-[11px] font-medium",
        toneClass[tone],
        className,
      )}
    >
      <span className="truncate">{children}</span>
    </span>
  );
}

export function ConnectionStatus({
  connected,
  label,
  status,
}: {
  connected?: boolean;
  label?: string;
  status?: "connected" | "connecting" | "disconnected" | "error" | "closed" | "unknown";
}) {
  const resolvedStatus = status ?? (connected ? "connected" : "disconnected");
  const tone: StatusTone =
    resolvedStatus === "connected"
      ? "success"
      : resolvedStatus === "connecting"
        ? "warning"
        : resolvedStatus === "error"
          ? "danger"
          : "neutral";

  return (
    <StatusBadge tone={tone}>
      {label ?? resolvedStatus}
    </StatusBadge>
  );
}
