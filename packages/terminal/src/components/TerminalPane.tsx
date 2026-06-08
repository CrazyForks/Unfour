import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { Terminal as XTerm } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import type { SshSessionEvent, SshSessionSummary } from "@unfour/command-client";
import { cn } from "@unfour/ui";
import { redactTerminalLog, useTerminalStore } from "../model/terminal-state";

export function TerminalPane({
  active,
  className,
  events,
  inputDisabled,
  readOnly,
  session,
}: {
  active?: boolean;
  className?: string;
  events: SshSessionEvent[];
  inputDisabled?: boolean;
  readOnly?: boolean;
  session: SshSessionSummary | null;
}) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const searchAddonRef = useRef<SearchAddon | null>(null);
  const renderedEventsRef = useRef(0);
  const renderedSessionIdRef = useRef<string | null>(null);

  const activeSessionId = useTerminalStore((s) => s.activeSessionId);
  const appendTerminalEvents = useTerminalStore((s) => s.appendTerminalEvents);
  const setTerminalSearchAddon = useTerminalStore((s) => s.setTerminalSearchAddon);

  // Mutable callback refs – updated in useEffect (not during render).
  const onSendInputRef = useRef<((data: string) => void) | null>(null);
  const onResizeRef = useRef<
    ((sessionId: string, cols: number, rows: number) => void) | null
  >(null);
  const appendRef = useRef(appendTerminalEvents);
  const activeSessionIdRef = useRef(activeSessionId);

  // Keep callback refs in sync with latest store / prop values.
  useEffect(() => {
    appendRef.current = appendTerminalEvents;
    activeSessionIdRef.current = activeSessionId;

    onSendInputRef.current =
      activeSessionId && !readOnly && !inputDisabled
        ? (data: string) => {
            import("@unfour/command-client").then(({ sendSshInput }) => {
              sendSshInput({
                workspaceId: session?.workspaceId ?? "",
                sessionId: activeSessionId,
                data,
              }).catch(() => {
                /* swallow – output stream still works */
              });
            });
          }
        : null;

    onResizeRef.current = activeSessionId
      ? (sessionId: string, cols: number, rows: number) => {
          import("@unfour/command-client").then(({ resizeSshSession }) => {
            resizeSshSession({
              workspaceId: session?.workspaceId ?? "",
              sessionId,
              cols,
              rows,
            }).catch(() => {
              /* resize failures are non-fatal */
            });
          });
        }
      : null;
  }, [
    activeSessionId,
    appendTerminalEvents,
    inputDisabled,
    readOnly,
    session?.workspaceId,
  ]);

  // ------------------------------------------------------------------
  // Terminal initialisation
  // ------------------------------------------------------------------

  useEffect(() => {
    if (!hostRef.current) {
      return;
    }

    const styles = getComputedStyle(document.documentElement);
    const token = (name: string) => styles.getPropertyValue(name).trim();
    const terminal = new XTerm({
      convertEol: true,
      cursorBlink: true,
      fontFamily: "JetBrains Mono, Consolas, ui-monospace, monospace",
      fontSize: 13,
      theme: {
        background: token("--u-color-terminal-bg"),
        cursor: token("--u-color-terminal-cursor"),
        foreground: token("--u-color-terminal-text"),
      },
    });
    const fitAddon = new FitAddon();
    const searchAddon = new SearchAddon();
    terminal.loadAddon(fitAddon);
    terminal.loadAddon(searchAddon);
    terminal.open(hostRef.current);
    safeFit(fitAddon);
    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;
    searchAddonRef.current = searchAddon;
    setTerminalSearchAddon(searchAddon);

    // ---------------------------------------------------------------
    // Capture keyboard input from xterm
    // ---------------------------------------------------------------
    const dataDisposable = terminal.onData((data: string) => {
      onSendInputRef.current?.(data);
    });

    // ---------------------------------------------------------------
    // Detect resize changes from FitAddon
    // ---------------------------------------------------------------
    let lastCols = terminal.cols;
    let lastRows = terminal.rows;
    const resizeDisposable = terminal.onResize(({ cols, rows }) => {
      if (cols !== lastCols || rows !== lastRows) {
        lastCols = cols;
        lastRows = rows;
        const sid = activeSessionIdRef.current;
        if (sid) {
          onResizeRef.current?.(sid, cols, rows);
        }
      }
    });

    // ---------------------------------------------------------------
    // Tauri event listener for streaming terminal output
    // ---------------------------------------------------------------
    let unlisten: (() => void) | null = null;
    listen<{ sessionId: string; data: string; closed?: boolean }>(
      "ssh://terminal-data",
      (event) => {
        const { sessionId, data, closed } = event.payload;
        if (closed) {
          appendRef.current([
            {
              sessionId,
              kind: "close",
              data: "SSH session disconnected.\r\n",
              createdAt: new Date().toISOString(),
            },
          ]);
          return;
        }
        if (data) {
          terminal.write(data);
          appendRef.current([
            {
              sessionId,
              kind: "output",
              data,
              createdAt: new Date().toISOString(),
            },
          ]);
        }
      },
    ).then((fn) => {
      unlisten = fn;
    });

    // ---------------------------------------------------------------
    // ResizeObserver for container size changes
    // ---------------------------------------------------------------
    const resizeObserver =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(() => {
            safeFit(fitAddon);
          });
    resizeObserver?.observe(hostRef.current);

    return () => {
      dataDisposable.dispose();
      resizeDisposable.dispose();
      resizeObserver?.disconnect();
      unlisten?.();
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
      searchAddonRef.current = null;
      renderedEventsRef.current = 0;
      setTerminalSearchAddon(null);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- one-time init
  }, []);

  // ------------------------------------------------------------------
  // Re-fit on active / session changes
  // ------------------------------------------------------------------

  useEffect(() => {
    const fitAddon = fitAddonRef.current;
    if (fitAddon) {
      window.requestAnimationFrame(() => safeFit(fitAddon));
    }
  }, [active, readOnly, session?.cols, session?.rows]);

  // ------------------------------------------------------------------
  // Render polling-based events (fallback for non-Tauri / mock mode)
  // ------------------------------------------------------------------

  useEffect(() => {
    const terminal = terminalRef.current;
    if (!terminal) {
      return;
    }

    if (renderedSessionIdRef.current !== (session?.sessionId ?? null)) {
      terminal.reset();
      renderedEventsRef.current = 0;
      renderedSessionIdRef.current = session?.sessionId ?? null;
    }

    if (events.length < renderedEventsRef.current) {
      terminal.reset();
      renderedEventsRef.current = 0;
    }

    if (events.length === 0 && renderedEventsRef.current === 0) {
      terminal.reset();
      terminal.write(
        session
          ? session.status === "active"
            ? `Connected to ${session.username}@${session.host}. Waiting for output.\r\n`
            : `Session ${session.username}@${session.host} is disconnected.\r\n`
          : "Select a connection and start a session.\r\n",
      );
      return;
    }

    const nextEvents = events.slice(renderedEventsRef.current);
    nextEvents.forEach((event) => {
      const data =
        event.kind === "input"
          ? `$ ${redactTerminalLog(event.data)}`
          : redactTerminalLog(event.data);
      terminal.write(data.endsWith("\r\n") ? data : `${data}\r\n`);
    });
    renderedEventsRef.current = events.length;
  }, [events, session]);

  return (
    <div
      className={cn(
        "flex min-h-0 min-w-0 flex-1 flex-col bg-[var(--u-color-terminal-bg)]",
        active && "ring-1 ring-inset ring-[var(--u-color-focus)]",
        className,
      )}
    >
      <div className="min-h-0 flex-1 overflow-hidden p-2" ref={hostRef} />
    </div>
  );
}

function safeFit(fitAddon: FitAddon) {
  try {
    fitAddon.fit();
  } catch {
    // The pane may be hidden during a shell resize. ResizeObserver retries once visible.
  }
}
