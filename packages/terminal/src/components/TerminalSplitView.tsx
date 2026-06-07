import type { SshSessionEvent, SshSessionSummary } from "@unfour/command-client";
import { SplitPane, cn } from "@unfour/ui";
import type { TerminalSplitMode } from "../model/types";
import { TerminalPane } from "./TerminalPane";

export function TerminalSplitView({
  activeSession,
  activeEvents,
  inputDisabled,
  inputPending,
  inputValue,
  onInputChange,
  onSendInput,
  secondaryEvents,
  secondarySession,
  splitMode,
}: {
  activeSession: SshSessionSummary | null;
  activeEvents: SshSessionEvent[];
  inputDisabled?: boolean;
  inputPending?: boolean;
  inputValue: string;
  onInputChange: (value: string) => void;
  onSendInput: () => void;
  secondaryEvents: SshSessionEvent[];
  secondarySession: SshSessionSummary | null;
  splitMode: TerminalSplitMode;
}) {
  const split = splitMode !== "single";

  const primaryPane = (
    <TerminalPane
      active
      events={activeEvents}
      inputDisabled={inputDisabled}
      inputPending={inputPending}
      inputValue={inputValue}
      onInputChange={onInputChange}
      onSendInput={onSendInput}
      session={activeSession}
    />
  );

  if (!split) {
    return primaryPane;
  }

  return (
    <SplitPane
      className="min-h-0 flex-1 bg-[var(--u-color-terminal-bg)]"
      defaultRatio={50}
      minPaneSize={180}
      orientation={splitMode === "horizontal" ? "vertical" : "horizontal"}
      resizable
    >
      {primaryPane}
      <div className={cn("flex min-h-0 min-w-0 flex-1")}>
        {secondarySession ? (
          <TerminalPane
            events={secondaryEvents}
            inputValue=""
            onInputChange={() => undefined}
            onSendInput={() => undefined}
            readOnly
            session={secondarySession}
          />
        ) : (
          <div className="flex min-h-0 flex-1 items-center justify-center bg-[var(--u-color-terminal-bg)] p-3 font-mono text-[12px] text-[var(--u-color-terminal-muted)]">
            Open another session to populate this pane.
          </div>
        )}
      </div>
    </SplitPane>
  );
}
