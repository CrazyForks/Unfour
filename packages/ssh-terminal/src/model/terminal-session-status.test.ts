import { describe, expect, it } from "vitest";
import type { SshSessionSummary } from "@unfour/command-client";
import { shouldRenderTerminalPane } from "./terminal-session-status";

function session(status: SshSessionSummary["status"]): SshSessionSummary {
  return {
    authKind: "password",
    cols: 120,
    connectionId: "connection-1",
    createdAt: "2026-01-01T00:00:00Z",
    host: "example.test",
    reconnectAttempt: 0,
    rows: 32,
    sessionId: `session-${status}`,
    status,
    updatedAt: "2026-01-01T00:00:00Z",
    username: "dev",
    workspaceId: "workspace-1",
  };
}

describe("terminal session status helpers", () => {
  it("renders live and reconnecting sessions in the terminal pane", () => {
    expect(shouldRenderTerminalPane(session("connected"))).toBe(true);
    expect(shouldRenderTerminalPane(session("degraded"))).toBe(true);
    expect(shouldRenderTerminalPane(session("reconnecting"))).toBe(true);
  });

  it("does not render empty failed or disconnected sessions as editable panes", () => {
    expect(shouldRenderTerminalPane(session("failed"))).toBe(false);
    expect(shouldRenderTerminalPane(session("disconnected"))).toBe(false);
    expect(shouldRenderTerminalPane(null)).toBe(false);
  });

  it("keeps failed or disconnected sessions with output available for log review", () => {
    expect(shouldRenderTerminalPane(session("failed"), 1)).toBe(true);
    expect(shouldRenderTerminalPane(session("disconnected"), 1)).toBe(true);
  });
});
