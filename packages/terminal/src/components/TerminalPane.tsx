import { useEffect, useRef } from "react";
import { Send } from "lucide-react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal as XTerm } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import type { SshSessionEvent, SshSessionSummary } from "@unfour/command-client";
import { Button, Input, cn } from "@unfour/ui";
import { redactTerminalLog } from "../model/terminal-state";

export function TerminalPane({
  active,
  className,
  events,
  inputDisabled,
  inputPending,
  inputValue,
  onInputChange,
  onSendInput,
  readOnly,
  session,
}: {
  active?: boolean;
  className?: string;
  events: SshSessionEvent[];
  inputDisabled?: boolean;
  inputPending?: boolean;
  inputValue: string;
  onInputChange: (value: string) => void;
  onSendInput: () => void;
  readOnly?: boolean;
  session: SshSessionSummary | null;
}) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const renderedEventsRef = useRef(0);
  const renderedSessionIdRef = useRef<string | null>(null);

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
    terminal.loadAddon(fitAddon);
    terminal.open(hostRef.current);
    safeFit(fitAddon);
    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;

    const resizeObserver =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(() => {
            safeFit(fitAddon);
          });
    resizeObserver?.observe(hostRef.current);

    return () => {
      resizeObserver?.disconnect();
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
      renderedEventsRef.current = 0;
    };
  }, []);

  useEffect(() => {
    const fitAddon = fitAddonRef.current;
    if (fitAddon) {
      window.requestAnimationFrame(() => safeFit(fitAddon));
    }
  }, [active, readOnly, session?.cols, session?.rows]);

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

  const disabled = readOnly || inputDisabled || !session || session.status !== "active";

  return (
    <div
      className={cn(
        "flex min-h-0 min-w-0 flex-1 flex-col bg-[var(--u-color-terminal-bg)]",
        active && "ring-1 ring-inset ring-[var(--u-color-focus)]",
        className,
      )}
    >
      <div className="min-h-0 flex-1 overflow-hidden p-2" ref={hostRef} />
      {!readOnly && (
        <div className="border-t border-[var(--u-color-terminal-border)] bg-[var(--u-color-terminal-input-bg)] p-2">
          <div className="flex min-w-0 gap-2">
            <Input
              className="border-[var(--u-color-terminal-input-border)] bg-[var(--u-color-terminal-input-bg)] font-mono text-[var(--u-color-terminal-text)] placeholder:text-[var(--u-color-terminal-muted)]"
              disabled={disabled}
              onChange={(event) => onInputChange(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
                  onSendInput();
                }
              }}
              placeholder="Command input"
              value={inputValue}
            />
            <Button
              disabled={disabled || !inputValue || inputPending}
              onClick={onSendInput}
              type="button"
            >
              <Send size={14} />
              Send
            </Button>
          </div>
        </div>
      )}
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
