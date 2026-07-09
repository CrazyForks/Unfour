import { Minus, Square, Copy, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { cn, usePlatform } from "@unfour/ui";
import { isTauriRuntime } from "./module-helpers";

export function WindowControls() {
  const isMac = usePlatform() === "macos";
  const isTauri = isTauriRuntime();
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = isTauri ? getCurrentWindow() : null;

  useEffect(() => {
    if (!appWindow) {
      return;
    }
    let unlisten: (() => void) | undefined;
    void appWindow.isMaximized().then(setIsMaximized);
    void appWindow
      .onResized(() => {
        void appWindow.isMaximized().then(setIsMaximized);
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
    };
  }, [appWindow]);

  // Browser preview: keep placeholder controls so the layout stays reviewable.
  if (!isTauri) {
    return (
      <div className="ml-1 flex items-center gap-1 text-[var(--u-color-text-soft)]">
        <Minus size={15} />
        <Square size={13} />
        <X size={15} />
      </div>
    );
  }

  // On macOS the native traffic lights handle window controls; custom buttons
  // would duplicate them.
  if (isMac) {
    return null;
  }

  return (
    <div className="ml-1 flex items-center">
      <TitlebarWindowButton
        ariaLabel="Minimize"
        icon={<Minus size={16} />}
        onClick={() => void appWindow!.minimize()}
      />
      <TitlebarWindowButton
        ariaLabel={isMaximized ? "Restore" : "Maximize"}
        icon={
          isMaximized ? (
            <Copy size={14} style={{ transform: "scaleX(-1)" }} />
          ) : (
            <Square size={14} />
          )
        }
        onClick={() => void appWindow!.toggleMaximize()}
      />
      <TitlebarWindowButton
        ariaLabel="Close"
        className="hover:bg-[var(--u-color-danger)] hover:text-[var(--u-color-text-on-color)]"
        icon={<X size={16} />}
        onClick={() => void appWindow!.close()}
      />
    </div>
  );
}

function TitlebarWindowButton({
  ariaLabel,
  className,
  icon,
  onClick,
}: {
  ariaLabel: string;
  className?: string;
  icon: React.ReactNode;
  onClick: () => void;
}) {
  return (
    <button
      aria-label={ariaLabel}
      className={cn(
        "flex h-8 w-10 items-center justify-center rounded-sm text-[var(--u-color-text-muted)] transition-colors hover:bg-[var(--u-color-surface-hover)] hover:text-[var(--u-color-text)]",
        className,
      )}
      onClick={onClick}
      type="button"
    >
      {icon}
    </button>
  );
}
