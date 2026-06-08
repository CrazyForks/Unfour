import { create } from "zustand";
import type { SshSessionEvent } from "@unfour/command-client";
import type { TerminalSplitMode } from "./types";

type SearchAddonLike = {
  findNext: (term: string) => boolean;
  findPrevious: (term: string) => boolean;
  clearDecorations: () => void;
  dispose: () => void;
};

type TerminalStore = {
  activeSessionId: string | null;
  exportedLog: string | null;
  searchOpen: boolean;
  searchQuery: string;
  splitMode: TerminalSplitMode;
  terminalEvents: SshSessionEvent[];
  terminalInput: string;
  terminalSearchAddon: SearchAddonLike | null;
  workspaceId: string | null;
  activateWorkspace: (workspaceId: string) => void;
  appendTerminalEvents: (events: SshSessionEvent[]) => void;
  clearTerminalSessionEvents: (sessionId: string | null) => void;
  resetTerminalEvents: () => void;
  setActiveSessionId: (sessionId: string | null) => void;
  setExportedLog: (content: string | null) => void;
  setSearchOpen: (open: boolean) => void;
  setSearchQuery: (query: string) => void;
  setSplitMode: (mode: TerminalSplitMode) => void;
  setTerminalSearchAddon: (addon: SearchAddonLike | null) => void;
  startTerminalSession: (sessionId: string, events: SshSessionEvent[]) => void;
  setTerminalEvents: (events: SshSessionEvent[]) => void;
  setTerminalInput: (input: string) => void;
};

export function defaultTerminalInput() {
  return "whoami\n";
}

export const useTerminalStore = create<TerminalStore>((set) => ({
  activeSessionId: null,
  exportedLog: null,
  searchOpen: false,
  searchQuery: "",
  splitMode: "single",
  terminalEvents: [],
  terminalInput: defaultTerminalInput(),
  terminalSearchAddon: null,
  workspaceId: null,
  activateWorkspace: (workspaceId) =>
    set((state) =>
      state.workspaceId === workspaceId
        ? state
        : {
            activeSessionId: null,
            exportedLog: null,
            searchOpen: false,
            searchQuery: "",
            splitMode: "single",
            terminalEvents: [],
            terminalInput: defaultTerminalInput(),
            terminalSearchAddon: state.terminalSearchAddon,
            workspaceId,
          },
    ),
  appendTerminalEvents: (events) =>
    set((state) => ({
      terminalEvents: [...state.terminalEvents, ...events],
    })),
  clearTerminalSessionEvents: (sessionId) =>
    set((state) => ({
      exportedLog: null,
      terminalEvents: sessionId
        ? state.terminalEvents.filter((event) => event.sessionId !== sessionId)
        : [],
    })),
  resetTerminalEvents: () =>
    set({
      activeSessionId: null,
      exportedLog: null,
      terminalEvents: [],
      terminalInput: defaultTerminalInput(),
    }),
  setActiveSessionId: (activeSessionId) => set({ activeSessionId }),
  setExportedLog: (exportedLog) => set({ exportedLog }),
  setSearchOpen: (searchOpen) => set({ searchOpen }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setSplitMode: (splitMode) => set({ splitMode }),
  setTerminalSearchAddon: (terminalSearchAddon) => set({ terminalSearchAddon }),
  startTerminalSession: (sessionId, events) =>
    set((state) => ({
      activeSessionId: sessionId,
      exportedLog: null,
      terminalEvents: [
        ...state.terminalEvents.filter((event) => event.sessionId !== sessionId),
        ...events,
      ],
    })),
  setTerminalEvents: (terminalEvents) => set({ terminalEvents }),
  setTerminalInput: (terminalInput) => set({ terminalInput }),
}));

export function redactTerminalLog(value: string) {
  return value
    .split(/\r?\n/)
    .map((line) => {
      if (
        /(^|\b)(authorization|cookie|proxy-authorization|x-api-key|x-auth-token|password|passphrase|private[-_ ]?key)(\b|:|=)/i.test(
          line,
        )
      ) {
        return "<redacted>";
      }

      return line;
    })
    .join("\n");
}
