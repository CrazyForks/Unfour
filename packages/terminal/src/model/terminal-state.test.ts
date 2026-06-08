import { describe, expect, it } from "vitest";
import { useTerminalStore } from "./terminal-state";

function resetStore() {
  useTerminalStore.setState({
    activeSessionId: null,
    exportedLog: null,
    searchOpen: false,
    searchQuery: "",
    splitMode: "single",
    terminalEvents: [],
    terminalInput: "whoami\n",
    terminalSearchAddon: null,
    workspaceId: null,
  });
}

describe("terminal-state store", () => {
  it("stores and clears the search addon reference", () => {
    resetStore();
    const store = useTerminalStore.getState();
    expect(store.terminalSearchAddon).toBeNull();

    const mockAddon = {
      findNext: () => true,
      findPrevious: () => false,
      clearDecorations: () => {},
      dispose: () => {},
    };
    store.setTerminalSearchAddon(mockAddon);
    expect(useTerminalStore.getState().terminalSearchAddon).toBe(mockAddon);

    store.setTerminalSearchAddon(null);
    expect(useTerminalStore.getState().terminalSearchAddon).toBeNull();
  });

  it("preserves search addon across workspace activation", () => {
    resetStore();
    const store = useTerminalStore.getState();

    const mockAddon = {
      findNext: () => true,
      findPrevious: () => false,
      clearDecorations: () => {},
      dispose: () => {},
    };
    store.setTerminalSearchAddon(mockAddon);
    store.activateWorkspace("ws-1");

    // Addon should be preserved when activating a workspace.
    expect(useTerminalStore.getState().terminalSearchAddon).toBe(mockAddon);
    expect(useTerminalStore.getState().workspaceId).toBe("ws-1");
  });

  it("appends terminal events from streaming", () => {
    resetStore();
    const store = useTerminalStore.getState();

    store.appendTerminalEvents([
      {
        sessionId: "s1",
        kind: "output",
        data: "line 1\r\n",
        createdAt: "2026-01-01T00:00:00Z",
      },
    ]);
    store.appendTerminalEvents([
      {
        sessionId: "s1",
        kind: "output",
        data: "line 2\r\n",
        createdAt: "2026-01-01T00:00:01Z",
      },
    ]);

    const events = useTerminalStore.getState().terminalEvents;
    expect(events).toHaveLength(2);
    expect(events[0].data).toBe("line 1\r\n");
    expect(events[1].data).toBe("line 2\r\n");
  });

  it("clears events for a specific session", () => {
    resetStore();
    const store = useTerminalStore.getState();

    store.appendTerminalEvents([
      {
        sessionId: "s1",
        kind: "output",
        data: "session 1",
        createdAt: "2026-01-01T00:00:00Z",
      },
      {
        sessionId: "s2",
        kind: "output",
        data: "session 2",
        createdAt: "2026-01-01T00:00:00Z",
      },
    ]);

    store.clearTerminalSessionEvents("s1");
    const events = useTerminalStore.getState().terminalEvents;
    expect(events).toHaveLength(1);
    expect(events[0].sessionId).toBe("s2");
  });

  it("toggles search open state", () => {
    resetStore();
    const store = useTerminalStore.getState();

    expect(store.searchOpen).toBe(false);
    store.setSearchOpen(true);
    expect(useTerminalStore.getState().searchOpen).toBe(true);
    store.setSearchOpen(false);
    expect(useTerminalStore.getState().searchOpen).toBe(false);
  });
});
