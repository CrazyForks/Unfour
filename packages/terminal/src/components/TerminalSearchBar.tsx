import { ChevronDown, ChevronUp, X } from "lucide-react";
import { IconButton, Input } from "@unfour/ui";
import { useTerminalSearch } from "../hooks/useTerminalSearch";

export function TerminalSearchBar() {
  const search = useTerminalSearch();

  if (!search.open) {
    return null;
  }

  return (
    <div className="absolute right-3 top-3 z-10 flex h-8 max-w-[min(420px,calc(100%-24px))] items-center gap-1 rounded-[var(--u-radius-md)] border border-[var(--u-color-border)] bg-[var(--u-color-surface)] px-1 shadow-sm">
      <Input
        aria-label="Search terminal output"
        className="h-6 w-52 border-transparent bg-transparent px-1"
        disabled
        onChange={(event) => search.setQuery(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            search.setOpen(false);
          }
        }}
        placeholder="Search integration pending"
        value={search.query}
      />
      <span className="min-w-8 text-center text-[11px] text-[var(--u-color-text-soft)]">
        unavailable
      </span>
      <IconButton disabled label="Previous terminal search result" size="compact">
        <ChevronUp size={13} />
      </IconButton>
      <IconButton disabled label="Next terminal search result" size="compact">
        <ChevronDown size={13} />
      </IconButton>
      <IconButton label="Close terminal search" onClick={() => search.setOpen(false)} size="compact">
        <X size={13} />
      </IconButton>
    </div>
  );
}
