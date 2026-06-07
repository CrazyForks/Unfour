import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import * as React from "react";
import { cn } from "./utils";

const menuContent =
  "z-50 min-w-[180px] overflow-hidden rounded-[var(--u-radius-md)] border border-[var(--u-color-border)] bg-[var(--u-color-surface)] p-1 text-[12px] text-[var(--u-color-text)] shadow-lg";
const menuItem =
  "flex h-7 cursor-default select-none items-center gap-2 rounded-[var(--u-radius-sm)] px-2 outline-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 data-[highlighted]:bg-[var(--u-color-surface-hover)]";

export const DropdownMenu = DropdownMenuPrimitive.Root;
export const DropdownMenuTrigger = DropdownMenuPrimitive.Trigger;
export function DropdownMenuContent({
  className,
  ...props
}: React.ComponentProps<typeof DropdownMenuPrimitive.Content>) {
  return <DropdownMenuPrimitive.Content className={cn(menuContent, className)} sideOffset={4} {...props} />;
}
export function DropdownMenuItem({
  className,
  ...props
}: React.ComponentProps<typeof DropdownMenuPrimitive.Item>) {
  return <DropdownMenuPrimitive.Item className={cn(menuItem, className)} {...props} />;
}

type ContextMenuState = {
  close: () => void;
  open: boolean;
  openAt: (position: { x: number; y: number }) => void;
  position: { x: number; y: number };
};

const ContextMenuContext = React.createContext<ContextMenuState | null>(null);

export function ContextMenu({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = React.useState(false);
  const [position, setPosition] = React.useState({ x: 0, y: 0 });

  React.useEffect(() => {
    if (!open) {
      return;
    }

    function close() {
      setOpen(false);
    }

    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        close();
      }
    }

    window.addEventListener("click", close);
    window.addEventListener("contextmenu", close);
    window.addEventListener("keydown", closeOnEscape);

    return () => {
      window.removeEventListener("click", close);
      window.removeEventListener("contextmenu", close);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [open]);

  const value = React.useMemo<ContextMenuState>(
    () => ({
      close: () => setOpen(false),
      open,
      openAt: (nextPosition) => {
        setPosition(nextPosition);
        setOpen(true);
      },
      position,
    }),
    [open, position],
  );

  return (
    <ContextMenuContext.Provider value={value}>
      {children}
    </ContextMenuContext.Provider>
  );
}

export function ContextMenuTrigger({
  asChild,
  children,
}: {
  asChild?: boolean;
  children: React.ReactElement<{ onContextMenu?: React.MouseEventHandler }>;
}) {
  const context = React.useContext(ContextMenuContext);

  if (!context) {
    return children;
  }

  const handleContextMenu: React.MouseEventHandler = (event) => {
    children.props.onContextMenu?.(event);
    if (event.defaultPrevented) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    context.openAt({ x: event.clientX, y: event.clientY });
  };

  if (asChild) {
    return React.cloneElement(children, { onContextMenu: handleContextMenu });
  }

  return (
    <span className="contents" onContextMenu={handleContextMenu}>
      {children}
    </span>
  );
}

export function ContextMenuContent({
  children,
  className,
}: {
  children?: React.ReactNode;
  className?: string;
}) {
  const context = React.useContext(ContextMenuContext);

  if (!context?.open) {
    return null;
  }

  return (
    <div
      className={cn(menuContent, className)}
      onClick={(event) => event.stopPropagation()}
      role="menu"
      style={{
        left: context.position.x,
        position: "fixed",
        top: context.position.y,
      }}
    >
      {children}
    </div>
  );
}

export function ContextMenuItem({
  children,
  className,
  disabled,
  onSelect,
}: {
  children?: React.ReactNode;
  className?: string;
  disabled?: boolean;
  onSelect?: () => void;
}) {
  const context = React.useContext(ContextMenuContext);

  return (
    <button
      className={cn("w-full text-left", menuItem, className)}
      disabled={disabled}
      onClick={() => {
        onSelect?.();
        context?.close();
      }}
      role="menuitem"
      type="button"
    >
      {children}
    </button>
  );
}
